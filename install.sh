#!/usr/bin/env bash
# install.sh — bootstrap cosmikase.
#
# One command to (re)build the environment: install prerequisites, install the
# chezmoi binary, then `chezmoi init --apply` this repo. chezmoi is the single
# orchestrator — it writes the dotfiles AND runs the run_onchange_ package
# scripts under chezmoi/ that read cosmikase.yaml.
#
# Usage:
#   ./install.sh                Full bootstrap (single sudo prompt up front).
#   ./install.sh --dry-run      Show what chezmoi would change; touch nothing.
#   ./install.sh --ci           CI mode (implies COSMIKASE_CI=1): apt core +
#                               chezmoi apply only. Skips flatpak, runtimes,
#                               AI tools, fonts and anything GUI/snap. Requires
#                               passwordless sudo. Used by the container test.
#
# Env:
#   COSMIKASE_CI=1              Same effect as --ci.
#   CHEZMOI_VERSION=vX.Y.Z      Pin the chezmoi binary version.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHEZMOI_SOURCE="$REPO_DIR/chezmoi"
LOCAL_BIN="$HOME/.local/bin"
CHEZMOI_VERSION="${CHEZMOI_VERSION:-v2.71.0}"

CI="${COSMIKASE_CI:-0}"
DRY_RUN=0
SUDO_KEEPALIVE_PID=""

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────
info() { printf '\033[1;36m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[warn]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[error]\033[0m %s\n' "$*" >&2; exit 1; }

usage() {
    # Print only the header comment block (lines 2-19), stripping the leading "# ".
    sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'
}

# ─────────────────────────────────────────────────────────────────────────────
# Argument parsing
# ─────────────────────────────────────────────────────────────────────────────
for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=1 ;;
        --ci)      CI=1 ;;
        -h|--help) usage; exit 0 ;;
        *) die "Unknown argument: $arg (try --help)" ;;
    esac
done
export COSMIKASE_CI="$CI"

# ─────────────────────────────────────────────────────────────────────────────
# OS check (warn only — the target is Pop!_OS 24.04+, but do not block)
# ─────────────────────────────────────────────────────────────────────────────
check_os() {
    if [ ! -r /etc/os-release ]; then
        warn "No /etc/os-release; cannot verify OS. Continuing."
        return
    fi
    # shellcheck disable=SC1091
    . /etc/os-release
    case " ${ID:-} ${ID_LIKE:-} " in
        *" pop "*|*" ubuntu "*|*" debian "*) : ;;
        *) warn "This is tuned for Pop!_OS/Ubuntu; found '${PRETTY_NAME:-unknown}'. Continuing." ;;
    esac
    local major="${VERSION_ID%%.*}"
    if [ -n "${VERSION_ID:-}" ] && [ "${major:-0}" -lt 24 ] 2>/dev/null; then
        warn "Detected ${PRETTY_NAME:-?}; cosmikase targets 24.04+. Continuing."
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Single sudo prompt + keep-alive (skipped in --dry-run: nothing is modified)
# ─────────────────────────────────────────────────────────────────────────────
ensure_sudo() {
    if [ "$CI" = "1" ]; then
        sudo -n true 2>/dev/null || die "CI mode requires passwordless sudo."
    else
        info "Requesting sudo access (single prompt)…"
        sudo -v || die "sudo is required to install apt packages."
    fi
    # Refresh the timestamp until this script exits.
    ( while true; do
        sudo -n true 2>/dev/null || exit
        sleep 50
        kill -0 "$$" 2>/dev/null || exit
      done ) &
    SUDO_KEEPALIVE_PID=$!
    trap 'sudo_cleanup' EXIT
}

sudo_cleanup() {
    if [ -n "$SUDO_KEEPALIVE_PID" ]; then
        kill "$SUDO_KEEPALIVE_PID" 2>/dev/null || true
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Prerequisites (curl, git, python3-yaml for the manifest reader)
# ─────────────────────────────────────────────────────────────────────────────
install_prereqs() {
    local pkgs=(curl git ca-certificates python3 python3-yaml)
    local missing=()
    local p
    for p in "${pkgs[@]}"; do
        dpkg -s "$p" >/dev/null 2>&1 || missing+=("$p")
    done
    if [ "${#missing[@]}" -eq 0 ]; then
        info "Prerequisites already present."
        return
    fi
    info "Installing prerequisites: ${missing[*]}"
    sudo apt-get update -qq
    sudo apt-get install -y "${missing[@]}"
}

# ─────────────────────────────────────────────────────────────────────────────
# chezmoi binary (pinned; installed to ~/.local/bin if missing)
# ─────────────────────────────────────────────────────────────────────────────
CHEZMOI=""
install_chezmoi() {
    if command -v chezmoi >/dev/null 2>&1; then
        CHEZMOI="$(command -v chezmoi)"
    elif [ -x "$LOCAL_BIN/chezmoi" ]; then
        CHEZMOI="$LOCAL_BIN/chezmoi"
    else
        info "Installing chezmoi $CHEZMOI_VERSION to $LOCAL_BIN"
        mkdir -p "$LOCAL_BIN"
        sh -c "$(curl -fsLS get.chezmoi.io)" -- -b "$LOCAL_BIN" -t "$CHEZMOI_VERSION"
        CHEZMOI="$LOCAL_BIN/chezmoi"
    fi
    info "Using chezmoi: $("$CHEZMOI" --version | head -1)"
}

# ─────────────────────────────────────────────────────────────────────────────
# Apply
# ─────────────────────────────────────────────────────────────────────────────
apply_chezmoi() {
    export PATH="$LOCAL_BIN:$PATH"
    local args=(init --source "$CHEZMOI_SOURCE" --apply)
    if [ "$DRY_RUN" = "1" ]; then
        args+=(--dry-run --verbose)
        info "Dry run — no changes will be made."
    fi
    "$CHEZMOI" "${args[@]}"
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────
main() {
    info "cosmikase bootstrap — repo: $REPO_DIR"
    [ "$CI" = "1" ] && info "CI mode: apt core + chezmoi apply only."
    check_os

    if [ "$DRY_RUN" = "1" ]; then
        # Dry run must not modify the system: skip sudo + apt, just render.
        install_chezmoi
        apply_chezmoi
        info "Dry run complete."
        return
    fi

    ensure_sudo
    install_prereqs
    install_chezmoi
    apply_chezmoi

    echo
    info "Done. Next: run the hardware preflight to check COSMIC + webcam readiness:"
    echo "    $REPO_DIR/bin/cosmikase-preflight"
    echo "    (or 'cosmikase' for the interactive menu)"
}

main "$@"
