"""Unified CLI for cosmikase.

This module provides a consolidated command-line interface for cosmikase,
combining theme management, configuration, and validation commands.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from cosmikase.chezmoi import update_chezmoi_data
from cosmikase.config import get_value, load_config
from cosmikase.schema import validate_config
from cosmikase.themes import discover_theme_dirs, find_theme_cli, list_themes


def cmd_theme(args: argparse.Namespace) -> int:
    """Switch to a different theme."""
    theme_dirs = discover_theme_dirs()
    if not theme_dirs:
        print("Error: No theme directories found", file=sys.stderr)
        return 1

    themes_dir = theme_dirs[0]
    available = list_themes(themes_dir)

    if args.list:
        print(f"Available themes in {themes_dir}:")
        for theme in available:
            print(f"  - {theme}")
        return 0

    if not args.name:
        print("Error: No theme name provided", file=sys.stderr)
        print(f"Available: {', '.join(available)}")
        return 1

    if args.name not in available:
        print(f"Error: Theme '{args.name}' not found", file=sys.stderr)
        print(f"Available: {', '.join(available)}")
        return 1

    # Update chezmoi configuration
    if not update_chezmoi_data(args.name, str(themes_dir)):
        print("Error: Failed to update chezmoi configuration", file=sys.stderr)
        return 1

    # Apply chezmoi
    if not args.no_apply:
        print("Applying dotfiles...")
        result = subprocess.run(["chezmoi", "apply", "--force"], capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Warning: chezmoi apply failed: {result.stderr}", file=sys.stderr)

    print(f"Theme '{args.name}' applied successfully!")

    # Run helper scripts if requested
    if not args.no_helpers:
        theme_cli = find_theme_cli()
        if theme_cli:
            # The shell script handles helper orchestration
            subprocess.run([theme_cli, args.name, "--no-chezmoi"], check=False)

    return 0


def cmd_config(args: argparse.Namespace) -> int:
    """Query configuration values."""
    config_path = Path(args.config)
    if not config_path.exists():
        print(f"Error: Config file not found: {config_path}", file=sys.stderr)
        return 1

    config = load_config(config_path)
    value = get_value(config, args.path, args.default)

    if isinstance(value, bool):
        print("true" if value else "false")
    elif value is not None:
        print(value)
    elif args.default:
        print(args.default)

    return 0


def cmd_validate(args: argparse.Namespace) -> int:
    """Validate configuration file."""
    is_valid, errors = validate_config(args.config)

    if is_valid:
        if not args.quiet:
            print(f"✓ Configuration is valid: {args.config}")
        return 0
    else:
        print(f"✗ Configuration errors in {args.config}:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1


def cmd_themes_dir(args: argparse.Namespace) -> int:
    """Show theme directories."""
    dirs = discover_theme_dirs()

    if not dirs:
        print("No theme directories found", file=sys.stderr)
        return 1

    if args.all:
        for d in dirs:
            print(d)
    else:
        print(dirs[0])

    return 0


def main(argv: list[str] | None = None) -> int:
    """Main entry point for the cosmikase CLI."""
    parser = argparse.ArgumentParser(
        prog="cosmikase",
        description="COSMIC Omakase - Pop!_OS workstation configuration",
    )
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # theme command
    theme_parser = subparsers.add_parser("theme", help="Switch themes")
    theme_parser.add_argument("name", nargs="?", help="Theme name to apply")
    theme_parser.add_argument("--list", "-l", action="store_true", help="List available themes")
    theme_parser.add_argument("--no-apply", action="store_true", help="Skip chezmoi apply")
    theme_parser.add_argument("--no-helpers", action="store_true", help="Skip helper scripts")
    theme_parser.set_defaults(func=cmd_theme)

    # config command
    config_parser = subparsers.add_parser("config", help="Query configuration")
    config_parser.add_argument("path", help="Dot-separated path (e.g., defaults.theme)")
    config_parser.add_argument("--config", "-c", default="cosmikase.yaml", help="Config file path")
    config_parser.add_argument("--default", "-d", default="", help="Default if not found")
    config_parser.set_defaults(func=cmd_config)

    # validate command
    validate_parser = subparsers.add_parser("validate", help="Validate configuration")
    validate_parser.add_argument(
        "config", nargs="?", default="cosmikase.yaml", help="Config file path"
    )
    validate_parser.add_argument("--quiet", "-q", action="store_true", help="Only print errors")
    validate_parser.set_defaults(func=cmd_validate)

    # themes-dir command
    themes_parser = subparsers.add_parser("themes-dir", help="Show theme directories")
    themes_parser.add_argument("--all", "-a", action="store_true", help="Show all directories")
    themes_parser.set_defaults(func=cmd_themes_dir)

    args = parser.parse_args(argv)

    if not args.command:
        parser.print_help()
        return 0

    return args.func(args)


def run() -> None:
    """Entry point for script invocation."""
    sys.exit(main())


if __name__ == "__main__":
    run()
