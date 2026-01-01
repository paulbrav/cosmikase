
import pytest
import yaml

from cosmikase.config import enabled_items, enabled_top_level, get_value, load_config, package_names


@pytest.fixture
def sample_config_dict():
    return {
        "defaults": {
            "theme": "nord",
            "install": True,
            "ghostty": True
        },
        "apt": {
            "core": [
                {"name": "fzf", "install": True},
                {"name": "steam-installer", "install": False},
                {"name": "zoxide"} # implicit True
            ]
        },
        "uv_tools": [
            "ruff",
            {"name": "mypy", "install": False},
            {"name": "ty", "install": True}
        ],
        "npm": [
            "@openai/codex",
            {"name": "@bitwarden/cli", "install": True}
        ]
    }

def test_load_config(tmp_path, sample_config_dict):
    config_file = tmp_path / "test-config.yaml"
    config_file.write_text(yaml.dump(sample_config_dict))
    
    loaded = load_config(config_file)
    assert loaded["defaults"]["theme"] == "nord"
    assert len(loaded["apt"]["core"]) == 3

def test_get_value(sample_config_dict):
    assert get_value(sample_config_dict, "defaults.theme") == "nord"
    assert get_value(sample_config_dict, "defaults.ghostty") is True
    assert get_value(sample_config_dict, "missing.path", "fallback") == "fallback"

def test_enabled_items(sample_config_dict):
    items = enabled_items(sample_config_dict, "apt", "core")
    assert len(items) == 2
    names = [i["name"] for i in items]
    assert "fzf" in names
    assert "zoxide" in names
    assert "steam-installer" not in names

def test_enabled_top_level(sample_config_dict):
    # uv_tools is a list
    items = enabled_top_level(sample_config_dict, "uv_tools")
    assert len(items) == 2
    names = [i["name"] for i in items]
    assert "ruff" in names
    assert "ty" in names
    assert "mypy" not in names

def test_package_names(sample_config_dict):
    names = package_names(sample_config_dict, "apt", "core")
    assert names == ["fzf", "zoxide"]

