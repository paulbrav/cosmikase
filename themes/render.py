#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["pyyaml>=6", "jinja2>=3"]
# ///
"""Render ghostty.conf and cosmic-term.ron for a theme from its palette.yaml.

palette.yaml is the declared source of truth for a theme's colours. The two
files this script generates are the ones whose colours are fully derivable from
the 7-key palette; every other theme file (cursor.json, cosmic.ron, btop, ...)
is hand-curated and left untouched.

Use this for NEW themes or to fix drift. Curated ghostty.conf / cosmic-term.ron
that carry richer, hand-tuned 16-colour palettes can stay as they are — the
renderer only reproduces the palette-defined colours (background, foreground,
cursor and the black/red/green/yellow ANSI slots), filling the rest with sane
fallbacks derived from the accent.

Usage:
    themes/render.py --theme nord           # write nord's two files
    themes/render.py --all                  # every theme with a palette.yaml
    themes/render.py --theme nord --out-dir /tmp/x   # render elsewhere (no overwrite)
    themes/render.py --all --dry-run        # print, don't write
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    import yaml
    from jinja2 import Environment, FileSystemLoader
except ImportError as exc:  # pragma: no cover - dependency guard
    sys.exit(
        f"error: missing dependency ({exc}). Run with 'uv run themes/render.py' "
        "or 'pip install pyyaml jinja2'."
    )

THEMES_DIR = Path(__file__).resolve().parent
TEMPLATES_DIR = THEMES_DIR / "_templates"
COLOR_KEYS = ["background", "foreground", "accent", "sidebar", "terminal", "error", "warning"]


def hex_to_rgb(hex_color: str) -> str:
    """'#RRGGBB' -> 'r, g, b, 1.0' with COSMIC's 8-decimal normalized floats."""
    h = hex_color.strip().lstrip("#")
    if len(h) != 6:
        raise ValueError(f"expected 6-digit hex colour, got {hex_color!r}")
    r, g, b = (int(h[i : i + 2], 16) / 255.0 for i in (0, 2, 4))
    return f"{r:.8f}, {g:.8f}, {b:.8f}, 1.0"


def load_palette(theme_dir: Path) -> dict:
    ppath = theme_dir / "palette.yaml"
    if not ppath.exists():
        raise FileNotFoundError(f"no palette.yaml in {theme_dir}")
    data = yaml.safe_load(ppath.read_text())
    colors = data.get("colors", {})
    missing = [k for k in COLOR_KEYS if k not in colors]
    if missing:
        raise ValueError(f"{theme_dir.name}: palette.yaml missing colours {missing}")
    return data


def build_context(palette: dict) -> dict:
    colors = palette["colors"]
    rgb = {k: hex_to_rgb(colors[k]) for k in COLOR_KEYS}
    # 16-slot ANSI array: the palette-defined colours in their canonical slots,
    # accent standing in for the blue/magenta/cyan the palette does not specify.
    order = [
        "sidebar",
        "error",
        "accent",
        "warning",
        "accent",
        "accent",
        "accent",
        "foreground",
        "sidebar",
        "error",
        "accent",
        "warning",
        "accent",
        "accent",
        "accent",
        "foreground",
    ]
    ansi = [rgb[k] for k in order]
    colors_block = "\n".join(f"        ({c})," for c in ansi)
    return {
        "name": palette.get("name", "Theme"),
        "ron_name": str(palette.get("name", "Theme")).replace(" ", "-"),
        "colors": colors,
        "rgb": rgb,
        "ansi": ansi,
        "colors_block": colors_block,
    }


def render_theme(theme_dir: Path, out_dir: Path, env: Environment, dry_run: bool) -> None:
    palette = load_palette(theme_dir)
    ctx = build_context(palette)
    out_dir.mkdir(parents=True, exist_ok=True)
    for template_name, out_name in (
        ("ghostty.conf.j2", "ghostty.conf"),
        ("cosmic-term.ron.j2", "cosmic-term.ron"),
    ):
        text = env.get_template(template_name).render(**ctx)
        if not text.endswith("\n"):
            text += "\n"
        if dry_run:
            print(f"----- {theme_dir.name}/{out_name} -----")
            print(text, end="")
        else:
            (out_dir / out_name).write_text(text)
            print(f"wrote {out_dir / out_name}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    grp = ap.add_mutually_exclusive_group(required=True)
    grp.add_argument("--theme", help="theme name (directory under themes/)")
    grp.add_argument("--all", action="store_true", help="render every theme with a palette.yaml")
    ap.add_argument("--out-dir", help="write into this dir instead of the theme dir")
    ap.add_argument("--dry-run", action="store_true", help="print instead of writing")
    args = ap.parse_args()

    if not TEMPLATES_DIR.is_dir():
        sys.exit(f"error: templates dir not found: {TEMPLATES_DIR}")
    env = Environment(
        loader=FileSystemLoader(str(TEMPLATES_DIR)),
        keep_trailing_newline=True,
        trim_blocks=False,
        lstrip_blocks=False,
    )

    if args.all:
        themes = sorted(
            d
            for d in THEMES_DIR.iterdir()
            if d.is_dir() and not d.name.startswith("_") and (d / "palette.yaml").exists()
        )
    else:
        theme_dir = THEMES_DIR / args.theme
        if not (theme_dir / "palette.yaml").exists():
            sys.exit(f"error: no palette.yaml for theme '{args.theme}'")
        themes = [theme_dir]

    for theme_dir in themes:
        out_dir = Path(args.out_dir) / theme_dir.name if args.out_dir else theme_dir
        render_theme(theme_dir, out_dir, env, args.dry_run)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
