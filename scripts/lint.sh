#!/usr/bin/env bash
# scripts/lint.sh — the single source of truth for which shell scripts are
# checked, consumed by both `make lint` and the CI ShellCheck job so the two
# can never disagree on membership or severity.
#
# Membership: install.sh, every sh/bash script under bin/ and scripts/, and the
# tests/*.sh helpers. The extensionless PEP 723 Python script
# bin/cosmikase-chezmoi (linted by ruff) and the bin/__pycache__ build artifact
# are excluded by the shebang check below — deliberately, not by a fragile
# `head -1 | grep 'sh$'` heuristic that also misses `#!/bin/sh -e`, matches
# comments ending in "sh", and would sweep in fish scripts.
set -euo pipefail

REPO_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"

# Shared severity floor for make and CI. Override with SHELLCHECK_SEVERITY.
SEVERITY="${SHELLCHECK_SEVERITY:-warning}"

# Return 0 if $1's first line is a POSIX sh / bash shebang.
is_shell_script() {
    local line interp
    line="$(head -n1 -- "$1" 2>/dev/null)" || return 1
    case "$line" in
        '#!'*) line="${line#\#!}" ;;
        *) return 1 ;;
    esac
    # Split the shebang into words; unwrap a leading `env [-flags] <interp>`.
    # shellcheck disable=SC2086
    set -- $line
    [ "$#" -gt 0 ] || return 1
    interp="$1"
    if [ "${interp##*/}" = "env" ]; then
        shift
        while [ "$#" -gt 0 ] && [ "${1#-}" != "$1" ]; do shift; done
        [ "$#" -gt 0 ] || return 1
        interp="$1"
    fi
    case "${interp##*/}" in
        sh | bash | dash | ksh) return 0 ;;
        *) return 1 ;;
    esac
}

scripts=(install.sh)

for f in bin/* scripts/*; do
    [ -f "$f" ] || continue
    if is_shell_script "$f"; then
        scripts+=("$f")
    fi
done

for f in tests/*.sh; do
    [ -f "$f" ] || continue
    scripts+=("$f")
done

echo "shellcheck --severity=$SEVERITY (${#scripts[@]} files): ${scripts[*]}"
exec shellcheck --severity="$SEVERITY" "${scripts[@]}"
