"""Safely update chezmoi configuration."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import tomli_w

# Use tomllib (standard library in 3.11+) or tomli for older versions
try:
    import tomllib
except ImportError:
    import tomli as tomllib


def update_chezmoi_data(theme: str, themes_dir: str) -> bool:
    """Safely update chezmoi.toml [data] section.

    Args:
        theme: Theme name to set.
        themes_dir: Directory containing themes.

    Returns:
        True if successful, False otherwise.
    """
    config_path = Path.home() / ".config" / "chezmoi" / "chezmoi.toml"

    # Ensure directory exists
    config_path.parent.mkdir(parents=True, exist_ok=True)

    data: dict[str, Any] = {}

    # Read existing config
    if config_path.exists():
        try:
            with open(config_path, "rb") as f:
                data = tomllib.load(f)
        except tomllib.TOMLDecodeError as e:
            print(f"Error: chezmoi.toml has invalid TOML syntax: {e}")
            print("Please fix the file manually or remove it to start fresh.")
            return False
        except PermissionError as e:
            print(f"Error: Cannot read chezmoi config (permission denied): {e}")
            return False
        except OSError as e:
            print(f"Error reading chezmoi config: {e}")
            return False

    # Update [data] section
    if "data" not in data:
        data["data"] = {}

    data["data"]["theme"] = theme
    data["data"]["themes_dir"] = themes_dir

    # Set some defaults if they don't exist
    defaults = {"font_family": "JetBrainsMono Nerd Font", "font_size": 9, "padding": 14}
    for key, val in defaults.items():
        if key not in data["data"]:
            data["data"][key] = val

    # Write atomically
    tmp_path = config_path.with_suffix(".tmp")
    try:
        with open(tmp_path, "wb") as f:
            tomli_w.dump(data, f)
        tmp_path.replace(config_path)
        return True
    except Exception as e:
        print(f"Error writing chezmoi config: {e}")
        if tmp_path.exists():
            tmp_path.unlink()
        return False


def _main() -> None:
    """CLI for updating chezmoi config from shell."""
    import argparse
    import sys

    parser = argparse.ArgumentParser(description="Update chezmoi configuration")
    parser.add_argument("theme", help="Theme name")
    parser.add_argument("themes_dir", help="Themes directory")

    args = parser.parse_args()
    if update_chezmoi_data(args.theme, args.themes_dir):
        sys.exit(0)
    else:
        sys.exit(1)


if __name__ == "__main__":
    _main()
