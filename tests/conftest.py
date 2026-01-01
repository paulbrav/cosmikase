"""Shared pytest fixtures for cosmikase tests."""

from pathlib import Path

import pytest
import yaml


@pytest.fixture
def tmp_themes_dir(tmp_path):
    """Create a temporary themes directory with sample themes."""
    themes_dir = tmp_path / "themes"
    themes_dir.mkdir()

    # Create sample themes
    for theme_name in ["nord", "tokyo-night", "catppuccin"]:
        theme_path = themes_dir / theme_name
        theme_path.mkdir()
        (theme_path / "backgrounds").mkdir()

        # Create theme.yaml
        manifest = {
            "name": theme_name.replace("-", " ").title(),
            "variant": "dark",
            "colors": {"background": "#000000", "foreground": "#ffffff"},
            "cursor": {"theme": theme_name.title(), "extension": f"ext.{theme_name}"},
        }
        with open(theme_path / "theme.yaml", "w") as f:
            yaml.dump(manifest, f)

    return themes_dir


@pytest.fixture
def sample_config_dict():
    """Return a sample cosmikase configuration dictionary."""
    return {
        "defaults": {"theme": "nord", "install": True, "ghostty": True},
        "apt": {
            "core": [
                {"name": "fzf", "install": True},
                {"name": "steam-installer", "install": False},
                {"name": "zoxide"},  # implicit True
            ]
        },
        "uv_tools": [
            "ruff",
            {"name": "mypy", "install": False},
            {"name": "ty", "install": True},
        ],
        "npm": ["@openai/codex", {"name": "@bitwarden/cli", "install": True}],
    }


@pytest.fixture
def sample_config_file(tmp_path, sample_config_dict):
    """Create a temporary config file and return its path."""
    config_file = tmp_path / "cosmikase.yaml"
    with open(config_file, "w") as f:
        yaml.dump(sample_config_dict, f)
    return config_file


@pytest.fixture
def mock_home(tmp_path, monkeypatch):
    """Mock Path.home() to return a temporary directory."""
    monkeypatch.setattr(Path, "home", lambda: tmp_path)
    return tmp_path


@pytest.fixture
def chezmoi_config_dir(mock_home):
    """Create the chezmoi config directory structure."""
    config_dir = mock_home / ".config" / "chezmoi"
    config_dir.mkdir(parents=True)
    return config_dir

