#!/usr/bin/env python3
"""Migrate legacy theme files to unified theme.yaml manifest."""

import json
from pathlib import Path
import yaml
import sys

# Add src to path to use discover_theme_dirs
sys.path.append(str(Path(__file__).resolve().parent.parent / "src"))
from cosmikase.themes import discover_theme_dirs


def migrate_theme(theme_path: Path):
    print(f"Migrating {theme_path.name}...")
    
    # Check if already migrated
    yaml_path = theme_path / "theme.yaml"
    if yaml_path.exists():
        print(f"  - theme.yaml already exists, skipping.")
        return

    # Gather data from legacy files
    is_light = (theme_path / "light.mode").exists()
    
    cursor_data = {}
    cursor_json = theme_path / "cursor.json"
    if cursor_json.exists():
        try:
            with open(cursor_json, "r") as f:
                cdata = json.load(f)
                cursor_data = {
                    "theme": cdata.get("colorTheme"),
                    "extension": cdata.get("extension")
                }
        except Exception as e:
            print(f"  - Error reading cursor.json: {e}")

    # Try to find a default wallpaper
    wallpaper = None
    bg_dir = theme_path / "backgrounds"
    if bg_dir.is_dir():
        # Prefer specific names if they exist, otherwise take first image
        v_images = sorted(list(bg_dir.glob("*.[jJ][pP][gG]")) + 
                          list(bg_dir.glob("*.[pP][nN][gG]")) +
                          list(bg_dir.glob("*.[wW][eE][bB][pP]")))
        if v_images:
            # Check for common default names
            defaults = ["default", "wallpaper", "background", "1-pop-default", "1-pop-fractal-mountains"]
            found = False
            for d in defaults:
                for img in v_images:
                    if d in img.name.lower():
                        wallpaper = f"backgrounds/{img.name}"
                        found = True
                        break
                if found: break
            if not found:
                wallpaper = f"backgrounds/{v_images[0].name}"

    # Build manifest
    manifest = {
        "name": theme_path.name.replace("-", " ").title(),
        "variant": "light" if is_light else "dark",
        "colors": {
            # Placeholder for common colors if we can extract them from RON files later
        }
    }
    
    if cursor_data:
        manifest["cursor"] = cursor_data
    
    if wallpaper:
        manifest["wallpaper"] = wallpaper

    # Write theme.yaml
    with open(yaml_path, "w") as f:
        yaml.dump(manifest, f, sort_keys=False, indent=2)
    
    print(f"  - Created theme.yaml")


def main():
    theme_dirs = discover_theme_dirs()
    if not theme_dirs:
        print("No theme directories found.")
        return

    for base_dir in theme_dirs:
        print(f"Searching for themes in {base_dir}...")
        for theme_path in base_dir.iterdir():
            if theme_path.is_dir() and not theme_path.name.startswith("."):
                migrate_theme(theme_path)


if __name__ == "__main__":
    main()
