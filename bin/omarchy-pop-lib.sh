#!/usr/bin/env bash
# Shared functions for omarchy-pop theme scripts
# This library is sourced by bin/ scripts and provides common utilities.

# Find themes directory using Python CLI (canonical source) with fallbacks
find_themes_dir() {
    # 1. Environment variable override
    if [[ -n "${THEMES_DIR:-}" ]] && [[ -d "$THEMES_DIR" ]]; then
        echo "$THEMES_DIR"
        return
    fi

    # 2. Use Python CLI if available (canonical implementation)
    if command -v omarchy-themes-dir >/dev/null 2>&1; then
        local py_result
        py_result=$(omarchy-themes-dir 2>/dev/null)
        if [[ -n "$py_result" ]] && [[ -d "$py_result" ]]; then
            echo "$py_result"
            return
        fi
    fi

    # 3. Try uv run if omarchy-themes-dir not in PATH
    if command -v uv >/dev/null 2>&1; then
        local py_result
        py_result=$(uv run omarchy-themes-dir 2>/dev/null)
        if [[ -n "$py_result" ]] && [[ -d "$py_result" ]]; then
            echo "$py_result"
            return
        fi
    fi

    # 4. Fallback: check common locations directly
    local installed="$HOME/.local/share/omarchy-pop/themes"
    if [[ -d "$installed" ]]; then
        echo "$installed"
        return
    fi

    # 5. Repo location (relative to this script)
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local repo_themes="$script_dir/../themes"
    if [[ -d "$repo_themes" ]]; then
        (cd "$repo_themes" && pwd)
        return
    fi

    # 6. Current working directory
    if [[ -d "./themes" ]]; then
        (cd "./themes" && pwd)
        return
    fi

    # Fallback to default location
    echo "$HOME/.local/share/omarchy-pop/themes"
}

# Find helper script - simplified since Ansible installs to ~/.local/bin
find_helper() {
    local name="$1"
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    # 1. Check script directory (for development/repo usage)
    if [[ -x "$script_dir/$name" ]]; then
        echo "$script_dir/$name"
        return
    fi

    # 2. Check ~/.local/bin (installed by Ansible)
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
        notify-send -a "omarchy-pop" "$title" "$message" 2>/dev/null || true
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

# Theme history management
HISTORY_FILE="$HOME/.config/omarchy-pop/theme-history"

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
