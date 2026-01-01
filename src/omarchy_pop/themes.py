"""Core theme logic and discovery for omarchy-pop."""

from __future__ import annotations

import json
import os
import shutil
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import yaml


@dataclass
class ThemeManifest:
    name: str
    variant: Literal["dark", "light"]
    colors: dict[str, str]
    cursor_theme: str | None = None
    cursor_extension: str | None = None
    wallpaper: str | None = None


def _unique_dirs(candidates: Iterable[Path]) -> list[Path]:
    seen = set()
    unique: list[Path] = []
    for path in candidates:
        resolved = path.expanduser()
        key = resolved.resolve() if resolved.exists() else resolved
        if key in seen or not resolved.is_dir():
            continue
        seen.add(key)
        unique.append(resolved)
    return unique


def _find_repo_root() -> Path | None:
    """Walk up from this file to find the repo root (contains themes/ and bin/)."""
    current = Path(__file__).resolve().parent
    for _ in range(5):  # Limit search depth
        if (current / "themes").is_dir() and (current / "bin").is_dir():
            return current
        if current.parent == current:
            break
        current = current.parent
    # Also check cwd
    cwd = Path.cwd()
    if (cwd / "themes").is_dir() and (cwd / "bin").is_dir():
        return cwd
    return None


def discover_theme_dirs() -> list[Path]:
    env_dir = os.environ.get("THEMES_DIR")
    repo_root = _find_repo_root()
    candidates = [
        Path(env_dir) if env_dir else None,
        repo_root / "themes" if repo_root else None,
        Path.cwd() / "themes",
        Path.home() / ".local" / "share" / "omarchy-pop" / "themes",
    ]
    return _unique_dirs(path for path in candidates if path is not None)


def list_themes(base: Path | None) -> list[str]:
    if base is None or not base.is_dir():
        return []
    return sorted(entry.name for entry in base.iterdir() if entry.is_dir())


def find_theme_cli() -> str | None:
    repo_root = _find_repo_root()
    candidates = [
        os.environ.get("THEME_CLI"),
        str(repo_root / "bin" / "omarchy-pop-theme") if repo_root else None,
        shutil.which("omarchy-pop-theme"),
        str(Path.home() / ".local" / "bin" / "omarchy-pop-theme"),
    ]
    for candidate in candidates:
        if not candidate:
            continue
        if shutil.which(candidate):
            return candidate
        path = Path(candidate).expanduser()
        if path.is_file() and os.access(path, os.X_OK):
            return str(path)
    return None


def load_manifest(theme_path: Path) -> ThemeManifest:
    """Load theme manifest from theme.yaml or fallback to legacy files."""
    yaml_path = theme_path / "theme.yaml"
    if yaml_path.exists():
        with open(yaml_path) as f:
            data = yaml.safe_load(f)
            return ThemeManifest(
                name=data.get("name", theme_path.name),
                variant=data.get("variant", "dark"),
                colors=data.get("colors", {}),
                cursor_theme=data.get("cursor", {}).get("theme"),
                cursor_extension=data.get("cursor", {}).get("extension"),
                wallpaper=data.get("wallpaper"),
            )

    # Fallback to legacy
    is_light = (theme_path / "light.mode").exists()
    
    # Try to load cursor.json if it exists
    cursor_json = theme_path / "cursor.json"
    cursor_theme = None
    cursor_extension = None
    if cursor_json.exists():
        try:
            with open(cursor_json) as f:
                cdata = json.load(f)
                cursor_theme = cdata.get("colorTheme")
                cursor_extension = cdata.get("extension")
        except Exception:
            pass

    return ThemeManifest(
        name=theme_path.name,
        variant="light" if is_light else "dark",
        colors={},
        cursor_theme=cursor_theme,
        cursor_extension=cursor_extension,
    )


def _main() -> None:
    """CLI entry point for theme directory discovery.
    
    Used by shell scripts to get the canonical themes directory.
    """
    import argparse
    import sys

    parser = argparse.ArgumentParser(
        description="Discover omarchy-pop theme directories"
    )
    parser.add_argument(
        "--all", "-a",
        action="store_true",
        help="Print all discovered theme directories (one per line)",
    )
    parser.add_argument(
        "--list", "-l",
        action="store_true",
        help="List available themes in the primary directory",
    )

    args = parser.parse_args()
    dirs = discover_theme_dirs()

    if not dirs:
        print("No theme directories found", file=sys.stderr)
        sys.exit(1)

    if args.all:
        for d in dirs:
            print(d)
    elif args.list:
        themes = list_themes(dirs[0])
        for theme in themes:
            print(theme)
    else:
        # Default: print primary theme directory
        print(dirs[0])


if __name__ == "__main__":
    _main()
