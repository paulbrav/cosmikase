"""Validation utilities for configuration files."""

from __future__ import annotations

from pathlib import Path


def validate_ron(path: Path | str) -> bool:
    """Basic RON syntax check - checks for balanced parentheses and brackets.

    This is not a full parser, but catches common syntax errors.
    """
    path = Path(path)
    if not path.exists():
        return False

    try:
        content = path.read_text()
    except Exception:
        return False

    stack = []
    pairs = {")": "(", "]": "[", "}": "{"}

    # Simple state machine to ignore content inside quotes
    in_quotes = False
    escaped = False

    for char in content:
        if escaped:
            escaped = False
            continue

        if char == "\\":
            escaped = True
            continue

        if char == '"':
            in_quotes = not in_quotes
            continue

        if in_quotes:
            continue

        if char in pairs.values():
            stack.append(char)
        elif char in pairs and (not stack or stack.pop() != pairs[char]):
            return False

    return len(stack) == 0


def _main() -> None:
    """CLI for RON validation."""
    import argparse
    import sys

    parser = argparse.ArgumentParser(description="Validate RON file syntax")
    parser.add_argument("path", help="Path to RON file")

    args = parser.parse_args()
    if validate_ron(args.path):
        print(f"File {args.path} is valid RON (basic check)")
        sys.exit(0)
    else:
        print(f"File {args.path} is NOT valid RON (basic check)")
        sys.exit(1)


if __name__ == "__main__":
    _main()
