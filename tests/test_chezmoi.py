"""Tests for the standalone bin/cosmikase-chezmoi script.

The old ``cosmikase.chezmoi`` Python package is gone; its atomic-TOML behaviour
now lives in the PEP 723 single-file script ``bin/cosmikase-chezmoi``. These
tests load that script two ways:

* by importing it from its path (``update_chezmoi_data`` unit tests), and
* by invoking it as a subprocess to lock the CLI contract that
  ``bin/cosmikase-theme`` depends on: ``cosmikase-chezmoi <theme>``.

Both paths require ``tomlkit`` to be importable (the test runner provides it,
e.g. ``uv run --with pytest,tomlkit pytest``); no network or ``uv`` is needed.
"""

import importlib.util
import os
import subprocess
import sys
from importlib.machinery import SourceFileLoader
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent
CHEZMOI_SCRIPT = REPO_ROOT / "bin" / "cosmikase-chezmoi"


def _load_chezmoi_module():
    """Import the extensionless bin/cosmikase-chezmoi script as a module."""
    loader = SourceFileLoader("cosmikase_chezmoi", str(CHEZMOI_SCRIPT))
    spec = importlib.util.spec_from_loader(loader.name, loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


chezmoi = _load_chezmoi_module()
update_chezmoi_data = chezmoi.update_chezmoi_data


class TestUpdateChezmoiData:
    """Unit tests for update_chezmoi_data (ported from the old module tests)."""

    @pytest.fixture
    def chezmoi_config(self, tmp_path, monkeypatch):
        """Point ``Path.home`` at a scratch dir and yield the chezmoi.toml path.

        Deliberately does NOT create the ``.config`` tree: several tests pin
        behaviour when ``~/.config`` does not yet exist (update_chezmoi_data owns
        creating the parent dir), and a mkdir-ing fixture would silently weaken
        them (item 6.7).
        """
        monkeypatch.setattr(Path, "home", lambda: tmp_path)
        return tmp_path / ".config" / "chezmoi" / "chezmoi.toml"

    def test_creates_new_config(self, chezmoi_config):
        """Creating a new chezmoi.toml when none exists."""
        result = update_chezmoi_data("nord")

        assert result is True
        assert chezmoi_config.exists()

        content = chezmoi_config.read_text()
        assert 'theme = "nord"' in content
        # Defaults are set on a fresh config.
        assert "font_family" in content
        assert "font_size" in content
        assert "padding" in content

    def test_updates_existing_config(self, chezmoi_config):
        """Updating an existing chezmoi.toml preserves other settings."""
        chezmoi_config.parent.mkdir(parents=True)
        chezmoi_config.write_text(
            "\n".join(
                [
                    "[data]",
                    'theme = "old-theme"',
                    'custom_setting = "keep-me"',
                    'font_family = "Custom Font"',
                    "",
                ]
            )
        )

        result = update_chezmoi_data("tokyo-night")

        assert result is True

        content = chezmoi_config.read_text()
        # New values are set.
        assert 'theme = "tokyo-night"' in content
        # Custom settings are preserved.
        assert 'custom_setting = "keep-me"' in content
        # An existing font_family is not overwritten with the default.
        assert 'font_family = "Custom Font"' in content

    def test_handles_malformed_toml(self, chezmoi_config, capsys):
        """Malformed TOML returns False and does not overwrite the file."""
        chezmoi_config.parent.mkdir(parents=True)
        original_content = "this is { not valid toml ["
        chezmoi_config.write_text(original_content)

        result = update_chezmoi_data("nord")

        assert result is False
        # File is left untouched.
        assert chezmoi_config.read_text() == original_content
        captured = capsys.readouterr()
        assert "invalid TOML syntax" in captured.out

    @pytest.mark.skipif(
        hasattr(os, "geteuid") and os.geteuid() == 0,
        reason="root bypasses filesystem permission checks",
    )
    def test_handles_permission_error(self, chezmoi_config, capsys):
        """Permission errors are handled gracefully (no crash, returns False)."""
        chezmoi_config.parent.mkdir(parents=True)
        chezmoi_config.write_text('[data]\ntheme = "old"\n')
        chezmoi_config.chmod(0o000)

        try:
            result = update_chezmoi_data("nord")
            assert result is False
            captured = capsys.readouterr()
            assert "permission denied" in captured.out.lower() or "Cannot read" in captured.out
        finally:
            chezmoi_config.chmod(0o644)

    def test_atomic_write(self, chezmoi_config):
        """A successful write leaves no temp file behind."""
        chezmoi_config.parent.mkdir(parents=True)
        chezmoi_config.write_text('[data]\ntheme = "original"\n')

        result = update_chezmoi_data("new-theme")
        assert result is True

        tmp_file = chezmoi_config.with_suffix(".tmp")
        assert not tmp_file.exists()

    def test_sets_defaults_on_new_config(self, chezmoi_config):
        """Default font/padding values are set on a brand-new config."""
        result = update_chezmoi_data("catppuccin")
        assert result is True

        content = chezmoi_config.read_text()

        assert 'font_family = "JetBrainsMono Nerd Font"' in content
        assert "font_size = 9" in content
        assert "padding = 14" in content


class TestCLIContract:
    """Subprocess tests locking the CLI that bin/cosmikase-theme invokes."""

    def _run(self, home, *args):
        env = {**os.environ, "HOME": str(home)}
        return subprocess.run(
            [sys.executable, str(CHEZMOI_SCRIPT), *args],
            capture_output=True,
            text=True,
            env=env,
        )

    def test_cli_single_positional_success(self, tmp_path):
        """`cosmikase-chezmoi <theme>` exits 0 and writes config."""
        result = self._run(tmp_path, "nord")
        assert result.returncode == 0, result.stderr

        config_path = tmp_path / ".config" / "chezmoi" / "chezmoi.toml"
        content = config_path.read_text()
        assert 'theme = "nord"' in content

    def test_cli_malformed_exits_nonzero(self, tmp_path):
        """A malformed existing config makes the CLI exit non-zero."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        (config_dir / "chezmoi.toml").write_text("this is { not valid toml [")

        result = self._run(tmp_path, "nord")
        assert result.returncode == 1

    def test_cli_missing_args_errors(self, tmp_path):
        """A missing theme positional produces an argparse usage error (exit 2)."""
        result = self._run(tmp_path)
        assert result.returncode == 2
        assert "theme" in result.stderr

    def test_cli_help(self, tmp_path):
        """`--help` succeeds and documents the theme positional."""
        result = self._run(tmp_path, "--help")
        assert result.returncode == 0
        assert "theme" in result.stdout
