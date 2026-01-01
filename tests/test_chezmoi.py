"""Tests for omarchy_pop.chezmoi module."""

import pytest
from pathlib import Path

from omarchy_pop.chezmoi import update_chezmoi_data


class TestUpdateChezmoiData:
    """Tests for the update_chezmoi_data function."""

    def test_creates_new_config(self, tmp_path, monkeypatch):
        """Test creating a new chezmoi.toml when none exists."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_path = config_dir / "chezmoi.toml"
        
        # Patch Path.home() to return our temp directory
        monkeypatch.setattr(Path, "home", lambda: tmp_path)
        
        result = update_chezmoi_data("nord", "/path/to/themes")
        
        assert result is True
        assert config_path.exists()
        
        content = config_path.read_text()
        assert 'theme = "nord"' in content
        assert 'themes_dir = "/path/to/themes"' in content
        # Check defaults are set
        assert "font_family" in content
        assert "font_size" in content
        assert "padding" in content

    def test_updates_existing_config(self, tmp_path, monkeypatch):
        """Test updating an existing chezmoi.toml preserves other settings."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        config_path = config_dir / "chezmoi.toml"
        
        # Create existing config with custom settings
        config_path.write_text("""
[data]
theme = "old-theme"
themes_dir = "/old/path"
custom_setting = "keep-me"
font_family = "Custom Font"
""")
        
        monkeypatch.setattr(Path, "home", lambda: tmp_path)
        
        result = update_chezmoi_data("tokyo-night", "/new/themes/path")
        
        assert result is True
        
        content = config_path.read_text()
        # New values should be set
        assert 'theme = "tokyo-night"' in content
        assert 'themes_dir = "/new/themes/path"' in content
        # Custom settings should be preserved
        assert 'custom_setting = "keep-me"' in content
        # Existing font_family should be preserved (not overwritten with default)
        assert 'font_family = "Custom Font"' in content

    def test_handles_malformed_toml(self, tmp_path, monkeypatch, capsys):
        """Test that malformed TOML returns False and doesn't overwrite."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        config_path = config_dir / "chezmoi.toml"
        
        # Create malformed TOML
        original_content = "this is { not valid toml ["
        config_path.write_text(original_content)
        
        monkeypatch.setattr(Path, "home", lambda: tmp_path)
        
        result = update_chezmoi_data("nord", "/path/to/themes")
        
        assert result is False
        # File should not be modified
        assert config_path.read_text() == original_content
        # Error message should be printed
        captured = capsys.readouterr()
        assert "invalid TOML syntax" in captured.out

    def test_handles_permission_error(self, tmp_path, monkeypatch, capsys):
        """Test that permission errors are handled gracefully."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        config_path = config_dir / "chezmoi.toml"
        
        # Create a valid config file
        config_path.write_text('[data]\ntheme = "old"')
        # Make it unreadable
        config_path.chmod(0o000)
        
        monkeypatch.setattr(Path, "home", lambda: tmp_path)
        
        try:
            result = update_chezmoi_data("nord", "/path/to/themes")
            assert result is False
            captured = capsys.readouterr()
            assert "permission denied" in captured.out.lower() or "Cannot read" in captured.out
        finally:
            # Restore permissions for cleanup
            config_path.chmod(0o644)

    def test_atomic_write(self, tmp_path, monkeypatch):
        """Test that writes are atomic (no partial writes on failure)."""
        config_dir = tmp_path / ".config" / "chezmoi"
        config_dir.mkdir(parents=True)
        config_path = config_dir / "chezmoi.toml"
        
        original_content = '[data]\ntheme = "original"'
        config_path.write_text(original_content)
        
        monkeypatch.setattr(Path, "home", lambda: tmp_path)
        
        # Successful update
        result = update_chezmoi_data("new-theme", "/themes")
        assert result is True
        
        # No .tmp file should remain
        tmp_file = config_path.with_suffix(".tmp")
        assert not tmp_file.exists()

    def test_sets_defaults_on_new_config(self, tmp_path, monkeypatch):
        """Test that default values are set when creating new config."""
        monkeypatch.setattr(Path, "home", lambda: tmp_path)
        
        result = update_chezmoi_data("catppuccin", "/themes")
        
        assert result is True
        
        config_path = tmp_path / ".config" / "chezmoi" / "chezmoi.toml"
        content = config_path.read_text()
        
        # Defaults should be present
        assert 'font_family = "JetBrainsMono Nerd Font"' in content
        assert "font_size = 9" in content
        assert "padding = 14" in content

