#!/usr/bin/env python3
"""
sync-theme-colors.py - Synchronize colors from cursor.json to all theme configs.

This script uses cursor.json as the source of truth and updates:
- antigravity.conf (Antigravity launcher)
- starship.toml (Starship prompt)
- nvim.lua / neovim.lua (Neovim colorscheme)
- ghostty.conf (Ghostty terminal)
- kitty.conf (Kitty terminal)
- alacritty.toml (Alacritty terminal)
- opencode.json (OpenCode AI)

Usage:
    uv run python scripts/sync-theme-colors.py           # Sync all themes
    uv run python scripts/sync-theme-colors.py tokyo-night  # Sync specific theme
"""
import argparse
import json
import re
import sys
from pathlib import Path


def hex_to_rgb(hex_color: str) -> tuple[float, float, float]:
    """Convert hex color (#RRGGBB) to normalized RGB tuple (0.0-1.0)."""
    hex_color = hex_color.lstrip("#")
    r = int(hex_color[0:2], 16) / 255.0
    g = int(hex_color[2:4], 16) / 255.0
    b = int(hex_color[4:6], 16) / 255.0
    return (r, g, b)


def update_theme(theme_path: Path, verbose: bool = True) -> bool:
    """Update all config files in a theme directory from cursor.json."""
    cursor_json_path = theme_path / "cursor.json"
    if not cursor_json_path.exists():
        if verbose:
            print(f"  Skipping {theme_path.name}: no cursor.json")
        return False

    try:
        with open(cursor_json_path, "r") as f:
            cursor_data = json.load(f)
    except json.JSONDecodeError as e:
        print(f"  Error reading {cursor_json_path}: {e}")
        return False

    colors = cursor_data.get("colors", {})
    if not colors:
        if verbose:
            print(f"  Skipping {theme_path.name}: no colors defined")
        return False

    # Extract core colors
    bg = colors.get("background")
    fg = colors.get("foreground")
    accent = colors.get("accent")
    sidebar = colors.get("sidebar", bg)
    terminal = colors.get("terminal", bg)
    error = colors.get("error")
    warning = colors.get("warning")

    if not all([bg, fg, accent]):
        if verbose:
            print(f"  Skipping {theme_path.name}: missing required colors (bg/fg/accent)")
        return False

    # Detect custom themes that have manually defined palettes
    is_custom = theme_path.name in [
        "cosmic-dark",
        "cosmic-light",
        "osaka-jade",
        "matte-black",
        "pop-default",
    ]

    updated = []

    # 1. Update antigravity.conf
    ag_path = theme_path / "antigravity.conf"
    if ag_path.exists():
        content = ag_path.read_text()
        content = re.sub(r"^background=.*", f"background={bg}", content, flags=re.MULTILINE)
        content = re.sub(r"^foreground=.*", f"foreground={fg}", content, flags=re.MULTILINE)
        content = re.sub(r"^accent=.*", f"accent={accent}", content, flags=re.MULTILINE)
        if error:
            content = re.sub(r"^error=.*", f"error={error}", content, flags=re.MULTILINE)
        if warning:
            content = re.sub(r"^warning=.*", f"warning={warning}", content, flags=re.MULTILINE)
        ag_path.write_text(content)
        updated.append("antigravity.conf")

    # 2. Update starship.toml
    starship_path = theme_path / "starship.toml"
    if starship_path.exists():
        content = starship_path.read_text()
        content = re.sub(r'^base = ".*"', f'base = "{bg}"', content, flags=re.MULTILINE)
        content = re.sub(r'^text = ".*"', f'text = "{fg}"', content, flags=re.MULTILINE)
        content = re.sub(r'^accent = ".*"', f'accent = "{accent}"', content, flags=re.MULTILINE)
        if error:
            content = re.sub(r'^err = ".*"', f'err = "{error}"', content, flags=re.MULTILINE)
        if warning:
            content = re.sub(r'^warn = ".*"', f'warn = "{warning}"', content, flags=re.MULTILINE)
        starship_path.write_text(content)
        updated.append("starship.toml")

    # 3. Update nvim.lua (or neovim.lua)
    for nvim_name in ["nvim.lua", "neovim.lua"]:
        nvim_path = theme_path / nvim_name
        if nvim_path.exists():
            content = nvim_path.read_text()
            content = re.sub(r'bg = ".*"', f'bg = "{bg}"', content)
            content = re.sub(r'fg = ".*"', f'fg = "{fg}"', content)
            content = re.sub(r'accent = ".*"', f'accent = "{accent}"', content)
            content = re.sub(r'subtle = ".*"', f'subtle = "{sidebar}"', content)
            if error:
                content = re.sub(r'error = ".*"', f'error = "{error}"', content)
            if warning:
                content = re.sub(r'warn = ".*"', f'warn = "{warning}"', content)
            nvim_path.write_text(content)
            updated.append(nvim_name)

    # 4. Update Ghostty
    ghostty_path = theme_path / "ghostty.conf"
    if ghostty_path.exists():
        content = ghostty_path.read_text()
        content = re.sub(
            r"^background = .*", f"background = {terminal}", content, flags=re.MULTILINE
        )
        content = re.sub(
            r"^foreground = .*", f"foreground = {fg}", content, flags=re.MULTILINE
        )
        content = re.sub(
            r"^palette = 0=.*", f"palette = 0={sidebar}", content, flags=re.MULTILINE
        )
        # For custom themes, also update accent/error/warning palette entries
        if is_custom:
            if accent:
                content = re.sub(
                    r"^palette = 2=.*", f"palette = 2={accent}", content, flags=re.MULTILINE
                )
            if error:
                content = re.sub(
                    r"^palette = 1=.*", f"palette = 1={error}", content, flags=re.MULTILINE
                )
            if warning:
                content = re.sub(
                    r"^palette = 3=.*", f"palette = 3={warning}", content, flags=re.MULTILINE
                )
        ghostty_path.write_text(content)
        updated.append("ghostty.conf")

    # 5. Update Kitty
    kitty_path = theme_path / "kitty.conf"
    if kitty_path.exists():
        content = kitty_path.read_text()
        content = re.sub(
            r"^background\s+.*", f"background    {terminal}", content, flags=re.MULTILINE
        )
        content = re.sub(
            r"^foreground\s+.*", f"foreground    {fg}", content, flags=re.MULTILINE
        )
        content = re.sub(
            r"^color0\s+.*", f"color0        {sidebar}", content, flags=re.MULTILINE
        )
        if is_custom:
            if accent:
                content = re.sub(
                    r"^color2\s+.*", f"color2        {accent}", content, flags=re.MULTILINE
                )
            if error:
                content = re.sub(
                    r"^color1\s+.*", f"color1        {error}", content, flags=re.MULTILINE
                )
            if warning:
                content = re.sub(
                    r"^color3\s+.*", f"color3        {warning}", content, flags=re.MULTILINE
                )
        kitty_path.write_text(content)
        updated.append("kitty.conf")

    # 6. Update Alacritty
    alacritty_path = theme_path / "alacritty.toml"
    if alacritty_path.exists():
        content = alacritty_path.read_text()
        content = re.sub(
            r'^background = ".*"', f'background = "{terminal}"', content, flags=re.MULTILINE
        )
        content = re.sub(
            r'^foreground = ".*"', f'foreground = "{fg}"', content, flags=re.MULTILINE
        )
        # Use more specific lookbehind to target normal black
        content = re.sub(r'(?<=black = ").*?(?=")', f"{sidebar}", content, count=1)
        if is_custom:
            if accent:
                content = re.sub(r'(?<=green = ").*?(?=")', f"{accent}", content, count=1)
            if error:
                content = re.sub(r'(?<=red = ").*?(?=")', f"{error}", content, count=1)
            if warning:
                content = re.sub(r'(?<=yellow = ").*?(?=")', f"{warning}", content, count=1)
        alacritty_path.write_text(content)
        updated.append("alacritty.toml")

    # 7. Update OpenCode
    opencode_path = theme_path / "opencode.json"
    if opencode_path.exists():
        try:
            with open(opencode_path, "r") as f:
                opencode_data = json.load(f)
            opencode_data["background"] = bg
            opencode_data["foreground"] = fg
            opencode_data["accent"] = accent
            if error:
                opencode_data["error"] = error
            if warning:
                opencode_data["warning"] = warning
            with open(opencode_path, "w") as f:
                json.dump(opencode_data, f, indent=2)
            updated.append("opencode.json")
        except (json.JSONDecodeError, KeyError) as e:
            print(f"  Warning: Could not update opencode.json: {e}")

    if verbose and updated:
        print(f"  Updated: {', '.join(updated)}")

    return True


def find_themes_dir() -> Path:
    """Find the themes directory relative to this script or from environment."""
    # Try script's parent directory first
    script_dir = Path(__file__).resolve().parent
    themes_dir = script_dir.parent / "themes"
    if themes_dir.exists():
        return themes_dir

    # Try current working directory
    cwd_themes = Path.cwd() / "themes"
    if cwd_themes.exists():
        return cwd_themes

    # Fallback to hardcoded path
    fallback = Path.home() / ".local" / "share" / "cosmikase" / "themes"
    if fallback.exists():
        return fallback

    raise FileNotFoundError("Could not find themes directory")


def main():
    parser = argparse.ArgumentParser(
        description="Synchronize colors from cursor.json to all theme configs."
    )
    parser.add_argument(
        "theme",
        nargs="?",
        help="Specific theme to sync (default: all themes)",
    )
    parser.add_argument(
        "-q",
        "--quiet",
        action="store_true",
        help="Suppress output",
    )
    args = parser.parse_args()

    try:
        themes_root = find_themes_dir()
    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    if not args.quiet:
        print(f"Themes directory: {themes_root}")

    if args.theme:
        # Sync specific theme
        theme_path = themes_root / args.theme
        if not theme_path.is_dir():
            print(f"Error: Theme '{args.theme}' not found in {themes_root}", file=sys.stderr)
            sys.exit(1)
        if not args.quiet:
            print(f"Syncing colors for theme: {args.theme}")
        update_theme(theme_path, verbose=not args.quiet)
    else:
        # Sync all themes
        count = 0
        for theme_dir in sorted(themes_root.iterdir()):
            if theme_dir.is_dir() and not theme_dir.name.startswith("_"):
                if not args.quiet:
                    print(f"Syncing: {theme_dir.name}")
                if update_theme(theme_dir, verbose=not args.quiet):
                    count += 1
        if not args.quiet:
            print(f"\nSynced {count} themes")


if __name__ == "__main__":
    main()
