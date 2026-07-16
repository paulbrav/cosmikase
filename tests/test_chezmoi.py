"""Tests for the standalone bin/cosmikase-chezmoi script.

The old ``cosmikase.chezmoi`` Python package is gone; its atomic-TOML behaviour
now lives in the PEP 723 single-file script ``bin/cosmikase-chezmoi``. These
tests load that script two ways:

* by importing it from its path (``update_chezmoi_data`` unit tests), and
* by invoking it as a subprocess to lock the CLI contract that
  ``bin/cosmikase-theme`` depends on: ``cosmikase-chezmoi <theme> <themes_dir>``.

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

    def test_creates_new_config(self, tmp_path, monkeypatch):
        """Creating a new chezmoi.toml when none exists."""
        config_path = tmp_path / ".config" / "chezmoi" / "chezmoi.toml"

        monkeypatch.setattr(Path, "home", lambda: tmp_path)

        result = update_chezmoi_data("nord", "/path/to/themes")

        assert result is True
        assert config_path.exists()

        content = config_path.read_text()
        assert 'theme = "nord"' in content
        assert 'themes_dir = "/path/to/themes"' in content
        # Defaults are set on a fresh config.
        assert "font_family" in content
        assert "font_size" in content
        assert "padding" in content

    def test_updates_existing_config(self, tmp_path, monkeypatch):
        """Updating an existing chezmoi.toml preserves other settings."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        config_path = config_dir / "chezmoi.toml"

        config_path.write_text(
            "\n".join(
                [
                    "[data]",
                    'theme = "old-theme"',
                    'themes_dir = "/old/path"',
                    'custom_setting = "keep-me"',
                    'font_family = "Custom Font"',
                    "",
                ]
            )
        )

        monkeypatch.setattr(Path, "home", lambda: tmp_path)

        result = update_chezmoi_data("tokyo-night", "/new/themes/path")

        assert result is True

        content = config_path.read_text()
        # New values are set.
        assert 'theme = "tokyo-night"' in content
        assert 'themes_dir = "/new/themes/path"' in content
        # Custom settings are preserved.
        assert 'custom_setting = "keep-me"' in content
        # An existing font_family is not overwritten with the default.
        assert 'font_family = "Custom Font"' in content

    def test_handles_malformed_toml(self, tmp_path, monkeypatch, capsys):
        """Malformed TOML returns False and does not overwrite the file."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        config_path = config_dir / "chezmoi.toml"

        original_content = "this is { not valid toml ["
        config_path.write_text(original_content)

        monkeypatch.setattr(Path, "home", lambda: tmp_path)

        result = update_chezmoi_data("nord", "/path/to/themes")

        assert result is False
        # File is left untouched.
        assert config_path.read_text() == original_content
        captured = capsys.readouterr()
        assert "invalid TOML syntax" in captured.out

    @pytest.mark.skipif(
        hasattr(os, "geteuid") and os.geteuid() == 0,
        reason="root bypasses filesystem permission checks",
    )
    def test_handles_permission_error(self, tmp_path, monkeypatch, capsys):
        """Permission errors are handled gracefully (no crash, returns False)."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        config_path = config_dir / "chezmoi.toml"

        config_path.write_text('[data]\ntheme = "old"\n')
        config_path.chmod(0o000)

        monkeypatch.setattr(Path, "home", lambda: tmp_path)

        try:
            result = update_chezmoi_data("nord", "/path/to/themes")
            assert result is False
            captured = capsys.readouterr()
            assert "permission denied" in captured.out.lower() or "Cannot read" in captured.out
        finally:
            config_path.chmod(0o644)

    def test_atomic_write(self, tmp_path, monkeypatch):
        """A successful write leaves no temp file behind."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        config_path = config_dir / "chezmoi.toml"

        config_path.write_text('[data]\ntheme = "original"\n')

        monkeypatch.setattr(Path, "home", lambda: tmp_path)

        result = update_chezmoi_data("new-theme", "/themes")
        assert result is True

        tmp_file = config_path.with_suffix(".tmp")
        assert not tmp_file.exists()

    def test_sets_defaults_on_new_config(self, tmp_path, monkeypatch):
        """Default font/padding values are set on a brand-new config."""
        monkeypatch.setattr(Path, "home", lambda: tmp_path)

        result = update_chezmoi_data("catppuccin", "/themes")
        assert result is True

        config_path = tmp_path / ".config" / "chezmoi" / "chezmoi.toml"
        content = config_path.read_text()

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

    def test_cli_two_positional_success(self, tmp_path):
        """`cosmikase-chezmoi <theme> <themes_dir>` exits 0 and writes config."""
        result = self._run(tmp_path, "nord", "/some/themes")
        assert result.returncode == 0, result.stderr

        config_path = tmp_path / ".config" / "chezmoi" / "chezmoi.toml"
        content = config_path.read_text()
        assert 'theme = "nord"' in content
        assert 'themes_dir = "/some/themes"' in content

    def test_cli_malformed_exits_nonzero(self, tmp_path):
        """A malformed existing config makes the CLI exit non-zero."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        (config_dir / "chezmoi.toml").write_text("this is { not valid toml [")

        result = self._run(tmp_path, "nord", "/themes")
        assert result.returncode == 1

    def test_cli_missing_args_errors(self, tmp_path):
        """Missing positional args produce an argparse usage error (exit 2)."""
        result = self._run(tmp_path, "only-one-arg")
        assert result.returncode == 2
        assert "themes_dir" in result.stderr

    def test_cli_help(self, tmp_path):
        """`--help` succeeds and documents both positional args."""
        result = self._run(tmp_path, "--help")
        assert result.returncode == 0
        assert "theme" in result.stdout
        assert "themes_dir" in result.stdout
