#!/usr/bin/env bash
# Shared functions for cosmikase theme scripts
# This library is sourced by bin/ scripts and provides common utilities.

# Find themes directory (env override, then installed and repo locations).
# The Python CLI that used to provide the canonical path was removed; discovery
# is now purely filesystem-based.
find_themes_dir() {
    # 1. Environment variable override
    if [[ -n "${THEMES_DIR:-}" ]] && [[ -d "$THEMES_DIR" ]]; then
        echo "$THEMES_DIR"
        return
    fi

    # 2. Installed location (chezmoi's symlink_ entries link the repo themes/ here)
    local installed="$HOME/.local/share/cosmikase/themes"
    if [[ -d "$installed" ]]; then
        echo "$installed"
        return
    fi

    # 3. Repo location (relative to this script)
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local repo_themes="$script_dir/../themes"
    if [[ -d "$repo_themes" ]]; then
        (cd "$repo_themes" && pwd)
        return
    fi

    # 4. Current working directory
    if [[ -d "./themes" ]]; then
        (cd "./themes" && pwd)
        return
    fi

    # Fallback to default location
    echo "$HOME/.local/share/cosmikase/themes"
}

# Find helper script (script dir, then ~/.local/bin, then PATH)
find_helper() {
    local name="$1"
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    # 1. Check script directory (for development/repo usage)
    if [[ -x "$script_dir/$name" ]]; then
        echo "$script_dir/$name"
        return
    fi

    # 2. Check ~/.local/bin (symlinked by chezmoi's native symlink_ entries)
    if [[ -x "$HOME/.local/bin/$name" ]]; then
        echo "$HOME/.local/bin/$name"
        return
    fi

    # 3. Check PATH
    if command -v "$name" >/dev/null 2>&1; then
        command -v "$name"
        return
    fi

    # Not found
    echo ""
}

# Resolve a cosmikase helper and run it, or report it missing (item 6.2). Uses
# an if/else, NOT `find_helper NAME && run || warn`: the &&/|| form misreports a
# helper that ran and exited non-zero as "not found". Returns the helper's exit
# status, or 1 when it is not installed.
run_helper() {
    local name="$1"
    shift
    local helper_path
    helper_path="$(find_helper "$name")"
    if [[ -n "$helper_path" ]]; then
        "$helper_path" "$@"
    else
        echo "$name not found"
        return 1
    fi
}

# Run an OPTIONAL cosmikase helper best-effort (item 6.2): resolve it with
# find_helper, run it (never failing the caller) if present, or print a single
# consistent skip line and continue if it is not installed. This is the sibling
# of run_helper for callers — chiefly the theme run_onchange hook — that must
# degrade rather than abort when a helper is absent. if/else, not `find_helper
# && run || echo skip`, so a helper that ran and exited non-zero is not
# misreported as skipped.
run_optional() {
    local name="$1"
    shift
    local helper_path
    helper_path="$(find_helper "$name")"
    if [[ -n "$helper_path" ]]; then
        "$helper_path" "$@" || true
    else
        echo "  - Skipping $name (helper not installed)"
    fi
}

# Logging function (respects QUIET variable)
log() {
    if [[ "${QUIET:-false}" != "true" ]]; then
        echo "$@"
    fi
}

# Desktop notification function
notify() {
    local title="$1"
    local message="$2"
    if command -v notify-send >/dev/null 2>&1; then
        notify-send -a "cosmikase" "$title" "$message" 2>/dev/null || true
    fi
}

# Read a top-level value from a JSON file (item 4.4). Prefers jq, falls back to
# python3, and prints an empty string for a missing key. The two backends spell
# booleans differently — jq's `// empty` prints lowercase `true`, python prints
# capitalised `True` — so callers reading a boolean flag must accept both (see
# the light-mode check in cosmikase-theme-cosmic).
parse_json() {
    local file="$1"
    local key="$2"
    if command -v jq >/dev/null 2>&1; then
        jq -r ".$key // empty" "$file"
    elif command -v python3 >/dev/null 2>&1; then
        python3 -c 'import json, sys; d = json.load(open(sys.argv[1])); print(d.get(sys.argv[2], ""))' "$file" "$key"
    else
        echo "Error: Neither jq nor python3 available for JSON parsing" >&2
        return 1
    fi
}

# Validate theme exists and set THEME_PATH
require_theme() {
    local theme="$1"
    local themes_dir="$2"

    if [[ -z "$theme" ]]; then
        echo "Error: No theme name provided" >&2
        return 1
    fi

    local theme_path="$themes_dir/$theme"
    if [[ ! -d "$theme_path" ]]; then
        echo "Error: Theme '$theme' not found in $themes_dir" >&2
        return 1
    fi

    # Used by scripts that source this file.
    # shellcheck disable=SC2034
    THEME_PATH="$theme_path"
    return 0
}

# Shared argument parser for the theme scripts (item 6.1). Consumes the arms that
# are byte-identical across cosmikase-theme, -cosmic and -cursor and sets the
# THEME and QUIET globals the callers read:
#     -h | --help     -> usage; exit 0
#     --quiet | -q    -> QUIET=true
#     <unknown -flag> -> error via usage; exit 1
#     <positional>    -> the single THEME name (error on a second)
# A script's divergent flags (--no-*, --rollback, --only …) are handled by an
# optional `parse_script_flag` hook it defines. The hook inspects "$1" (and "$2"
# for value-taking flags), mutates its own option globals, sets SHIFT_COUNT to
# the number of args it consumed (defaults to 1 — only multi-arg flags override
# it) and returns 0; it returns non-zero when "$1" is not one of its flags. The
# caller must define usage() before calling this.
parse_common_args() {
    # THEME and QUIET are consumed by the sourcing script.
    # shellcheck disable=SC2034
    THEME=""
    QUIET=false
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                usage
                exit 0
                ;;
            --quiet|-q)
                QUIET=true
                shift
                ;;
            *)
                if declare -F parse_script_flag >/dev/null 2>&1; then
                    SHIFT_COUNT=1
                    if parse_script_flag "$@"; then
                        shift "$SHIFT_COUNT"
                        continue
                    fi
                fi
                case "$1" in
                    -*)
                        echo "Unknown option: $1" >&2
                        usage
                        exit 1
                        ;;
                    *)
                        if [[ -z "$THEME" ]]; then
                            THEME="$1"
                        else
                            echo "Error: Multiple theme names provided" >&2
                            usage
                            exit 1
                        fi
                        shift
                        ;;
                esac
                ;;
        esac
    done
}

# Theme history management
HISTORY_FILE="$HOME/.config/cosmikase/theme-history"

save_theme_history() {
    local theme="$1"
    mkdir -p "$(dirname "$HISTORY_FILE")"

    # Don't record if it's the same as last one
    local last
    last=$(tail -1 "$HISTORY_FILE" 2>/dev/null || echo "")
    if [[ "$theme" == "$last" ]]; then
        return
    fi

    echo "$theme" >> "$HISTORY_FILE"
    # Keep last 20 entries
    local tmp
    tmp=$(mktemp)
    tail -20 "$HISTORY_FILE" > "$tmp" && mv "$tmp" "$HISTORY_FILE"
}

get_previous_theme() {
    if [[ ! -f "$HISTORY_FILE" ]]; then
        return 1
    fi
    # Current theme is last line, so we want the one before it
    tail -2 "$HISTORY_FILE" | head -1
}
