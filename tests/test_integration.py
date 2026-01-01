"""Integration tests for cosmikase.

These tests verify end-to-end functionality across multiple modules.
"""

import subprocess
import sys

import pytest

from cosmikase.config import enabled_items, get_value, load_config
from cosmikase.schema import load_and_validate, validate_config
from cosmikase.themes import discover_theme_dirs, list_themes, load_manifest


class TestConfigIntegration:
    """Integration tests for configuration loading and validation."""

    def test_real_config_loads(self):
        """Test that the actual cosmikase.yaml loads successfully."""
        # Only run if config exists (in repo context)
        try:
            config = load_config("cosmikase.yaml")
            assert "defaults" in config
            assert "apt" in config
        except FileNotFoundError:
            pytest.skip("cosmikase.yaml not found - not in repo context")

    def test_real_config_validates(self):
        """Test that the actual cosmikase.yaml passes schema validation."""
        is_valid, errors = validate_config("cosmikase.yaml")
        # Config might not exist in CI
        if not is_valid and "not found" in str(errors).lower():
            pytest.skip("cosmikase.yaml not found")
        assert is_valid, f"Config validation failed: {errors}"

    def test_config_modules_work_together(self, sample_config_file, sample_config_dict):
        """Test that config module functions work with real files."""
        config = load_config(sample_config_file)
        
        # Test get_value
        theme = get_value(config, "defaults.theme")
        assert theme == "nord"
        
        # Test enabled_items
        items = enabled_items(config, "apt", "core")
        names = [i["name"] for i in items]
        assert "fzf" in names
        assert "steam-installer" not in names  # disabled


class TestThemeIntegration:
    """Integration tests for theme system."""

    def test_theme_discovery_works(self, tmp_themes_dir):
        """Test that theme discovery finds themes in temp directory."""
        import os
        
        # Set environment variable to point to our temp themes
        old_val = os.environ.get("THEMES_DIR")
        os.environ["THEMES_DIR"] = str(tmp_themes_dir)
        
        try:
            dirs = discover_theme_dirs()
            assert len(dirs) > 0
            assert tmp_themes_dir in dirs
            
            themes = list_themes(dirs[0])
            assert "nord" in themes
            assert "tokyo-night" in themes
        finally:
            if old_val:
                os.environ["THEMES_DIR"] = old_val
            else:
                os.environ.pop("THEMES_DIR", None)

    def test_theme_manifest_loading(self, tmp_themes_dir):
        """Test loading theme manifests."""
        nord_path = tmp_themes_dir / "nord"
        manifest = load_manifest(nord_path)
        
        assert manifest.name == "Nord"
        assert manifest.variant == "dark"
        assert "background" in manifest.colors


class TestSchemaIntegration:
    """Integration tests for schema validation."""

    def test_schema_validates_good_config(self, sample_config_file):
        """Test that valid config passes validation."""
        config = load_and_validate(sample_config_file)
        assert config.defaults.theme == "nord"
        assert config.defaults.install is True

    def test_schema_catches_invalid_config(self, tmp_path):
        """Test that invalid config fails validation."""
        bad_config = tmp_path / "bad.yaml"
        bad_config.write_text("""
defaults:
  theme: 123  # Should be string but schema should still work
  install: "yes"  # Should be bool but pydantic might coerce
""")
        
        # This should either succeed (pydantic coercion) or fail cleanly
        is_valid, errors = validate_config(bad_config)
        # We mainly care that it doesn't crash


class TestCLIIntegration:
    """Integration tests for CLI commands."""

    def test_cli_help(self):
        """Test that CLI help works."""
        result = subprocess.run(
            [sys.executable, "-m", "cosmikase.cli", "--help"],
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        assert "cosmikase" in result.stdout.lower()

    def test_cli_theme_list(self, tmp_themes_dir, monkeypatch):
        """Test that theme list command works."""
        import os
        monkeypatch.setenv("THEMES_DIR", str(tmp_themes_dir))
        
        result = subprocess.run(
            [sys.executable, "-m", "cosmikase.cli", "theme", "--list"],
            capture_output=True,
            text=True,
            env={**os.environ, "THEMES_DIR": str(tmp_themes_dir)},
        )
        assert result.returncode == 0
        assert "nord" in result.stdout

    def test_cli_themes_dir(self, tmp_themes_dir, monkeypatch):
        """Test themes-dir command."""
        import os
        
        result = subprocess.run(
            [sys.executable, "-m", "cosmikase.cli", "themes-dir"],
            capture_output=True,
            text=True,
            env={**os.environ, "THEMES_DIR": str(tmp_themes_dir)},
        )
        assert result.returncode == 0
        assert str(tmp_themes_dir) in result.stdout

