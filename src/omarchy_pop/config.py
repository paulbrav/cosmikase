"""Configuration helpers for omarchy-pop.

Provides YAML config parsing utilities for shell scripts and Python tools.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import yaml


def load_config(path: Path | str) -> dict[str, Any]:
    """Load and parse the omarchy-pop YAML configuration file."""
    path = Path(path)
    return yaml.safe_load(path.read_text())


def enabled_items(config: dict, section: str, group: str) -> list[dict[str, Any]]:
    """Return items from config[section][group] where install=true.

    Args:
        config: Parsed YAML configuration dict.
        section: Top-level section (e.g., 'apt', 'flatpak', 'installers').
        group: Sub-group within section (e.g., 'core', 'utility', 'runtimes').

    Returns:
        List of item dicts that have install=true (or install not specified).
    """
    return [
        item
        for item in config.get(section, {}).get(group, [])
        if item.get("install", True)
    ]


def enabled_top_level(config: dict, section: str) -> list[dict[str, Any]]:
    """Return items from config[section] (top-level list) where install=true.

    Useful for sections like 'uv_tools' or 'npm' that are direct lists.
    """
    items = config.get(section, [])
    if not isinstance(items, list):
        return []
    return [
        item if isinstance(item, dict) else {"name": item}
        for item in items
        if not isinstance(item, dict) or item.get("install", True)
    ]


def get_value(config: dict, dotpath: str, default: Any = None) -> Any:
    """Get nested value using dot notation like 'defaults.theme'.

    Args:
        config: Parsed YAML configuration dict.
        dotpath: Dot-separated path (e.g., 'defaults.theme', 'hp_zbook_ultra.emit_notes').
        default: Value to return if path doesn't exist.

    Returns:
        The value at the path, or default if not found.
    """
    cur: Any = config
    for part in dotpath.split("."):
        if isinstance(cur, dict):
            cur = cur.get(part)
        else:
            return default
    return cur if cur is not None else default


def package_names(config: dict, section: str, group: str) -> list[str]:
    """Extract package names from enabled items.

    Handles both 'name' and 'id' keys (for apt vs flatpak).
    """
    return [
        item.get("name") or item.get("id")
        for item in enabled_items(config, section, group)
        if item.get("name") or item.get("id")
    ]


def to_json(config: dict, section: str, group: str | None = None) -> str:
    """Output enabled items as JSON for shell consumption."""
    if group:
        items = enabled_items(config, section, group)
    else:
        items = enabled_top_level(config, section)
    return json.dumps(items, indent=2)


def _main() -> None:
    """CLI entry point for shell scripts to query config."""
    import argparse
    import sys

    parser = argparse.ArgumentParser(
        description="Query omarchy-pop YAML configuration"
    )
    parser.add_argument(
        "--config",
        "-c",
        default="omarchy-pop.yaml",
        help="Path to config file (default: omarchy-pop.yaml)",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # get command
    get_parser = subparsers.add_parser("get", help="Get a value by dotpath")
    get_parser.add_argument("path", help="Dot-separated path (e.g., defaults.theme)")
    get_parser.add_argument("--default", "-d", default="", help="Default if not found")

    # list command
    list_parser = subparsers.add_parser("list", help="List enabled items")
    list_parser.add_argument("section", help="Section name (e.g., apt, flatpak)")
    list_parser.add_argument("group", nargs="?", help="Group name (e.g., core, utility)")
    list_parser.add_argument(
        "--names-only", "-n", action="store_true", help="Output only package names"
    )
    list_parser.add_argument(
        "--json", "-j", action="store_true", help="Output as JSON"
    )

    args = parser.parse_args()
    config_path = Path(args.config)
    if not config_path.exists():
        print(f"Config file not found: {config_path}", file=sys.stderr)
        sys.exit(1)

    config = load_config(config_path)

    if args.command == "get":
        value = get_value(config, args.path, args.default)
        if isinstance(value, bool):
            print("true" if value else "false")
        else:
            print(value)

    elif args.command == "list":
        if args.json:
            print(to_json(config, args.section, args.group))
        elif args.names_only:
            if args.group:
                names = package_names(config, args.section, args.group)
            else:
                items = enabled_top_level(config, args.section)
                names = [item.get("name", "") for item in items if item.get("name")]
            for name in names:
                print(name)
        else:
            if args.group:
                items = enabled_items(config, args.section, args.group)
            else:
                items = enabled_top_level(config, args.section)
            for item in items:
                name = item.get("name") or item.get("id") or ""
                desc = item.get("desc", "")
                if desc:
                    print(f"{name}: {desc}")
                else:
                    print(name)


if __name__ == "__main__":
    _main()

