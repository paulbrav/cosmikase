#!/usr/bin/env bash
# container-smoke-assert.sh - assertions run INSIDE the smoke-test container.
#
# By the time this runs, the Containerfile has already executed
# `COSMIKASE_CI=1 ./install.sh` as the non-root `tester` user, so chezmoi has
# been applied and the apt "core" packages installed (runtimes/flatpak/GUI
# steps are skipped in CI mode). This script verifies the machine was actually
# built and that bin/cosmikase-preflight degrades cleanly with no COSMIC/webcam.
#
# Exit 0 iff every assertion passes.
set -uo pipefail

REPO_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

fail=0
pass() { echo "PASS: $*"; }
fatal() {
    echo "FAIL: $*"
    fail=1
}

echo "=== cosmikase container smoke assertions ==="
echo "repo: $REPO_DIR"
echo "home: $HOME"
echo

# 1. chezmoi applied ~/.bashrc with expected content.
#    starship + zoxide blocks are guaranteed by the bashrc template.
if [[ -f "$HOME/.bashrc" ]] && grep -Eq 'starship|zoxide' "$HOME/.bashrc"; then
    pass "HOME/.bashrc applied by chezmoi (contains starship/zoxide block)"
else
    fatal "HOME/.bashrc missing or lacks expected chezmoi-managed content"
fi

# 2. chezmoi applied the ghostty config directory.
if [[ -d "$HOME/.config/ghostty" ]]; then
    pass "HOME/.config/ghostty exists"
else
    fatal "HOME/.config/ghostty was not created by chezmoi apply"
fi

# 3. apt prerequisites installed by install.sh.
for pkg in git curl; do
    if dpkg -s "$pkg" >/dev/null 2>&1; then
        pass "apt prerequisite present: $pkg"
    else
        fatal "apt prerequisite missing: $pkg"
    fi
done

# 4. apt "core" packages installed (proves the package engine ran, not just
#    the bootstrap prereqs). Require at least one modern CLI from the roster.
core_candidates=(fzf zoxide bat eza ripgrep fd-find jq btop)
found_core=""
for pkg in "${core_candidates[@]}"; do
    if dpkg -s "$pkg" >/dev/null 2>&1; then
        found_core="$pkg"
        break
    fi
done
if [[ -n "$found_core" ]]; then
    pass "apt core package present (e.g. $found_core)"
else
    fatal "no apt core packages found (expected one of: ${core_candidates[*]})"
fi

# 5. Theme layer actually applied (not a silent no-op). The
#    run_onchange_after_10-setup-theme hook copies the active theme's btop.theme
#    to ~/.config/btop/themes/<theme>.theme. The hook bakes an absolute,
#    homeDir-based THEMES_DIR at render time; if that dir is missing the hook
#    exits early and this file is absent — so its presence proves the hook found
#    the themes dir and did real work. The CI default theme is "nord"
#    (COSMIKASE_CI skips the prompt), so nord.theme is expected.
btop_theme_dir="$HOME/.config/btop/themes"
if compgen -G "$btop_theme_dir/*.theme" >/dev/null 2>&1; then
    applied_theme="$(basename "$(compgen -G "$btop_theme_dir/*.theme" | head -1)")"
    pass "theme hook applied a btop theme ($applied_theme in $btop_theme_dir)"
else
    fatal "no btop theme applied (expected $btop_theme_dir/nord.theme) — theme hook silently no-op'd (themes dir missing?)"
fi

# 6. bin/cosmikase-preflight runs and exits nonzero-but-cleanly.
#    A container has no COSMIC session and no webcam firmware, so preflight
#    should report FAIL/WARN rows and exit nonzero WITHOUT crashing
#    (i.e. not a shell "command not found"/"not executable"/signal death).
preflight="$REPO_DIR/bin/cosmikase-preflight"
if [[ -x "$preflight" ]]; then
    preflight_out="$("$preflight" 2>&1)"
    rc=$?
    if [[ $rc -ne 0 && $rc -lt 126 && -n "$preflight_out" ]]; then
        pass "cosmikase-preflight ran and exited cleanly (rc=$rc)"
    else
        fatal "cosmikase-preflight rc=$rc (want: nonzero, <126, with output)"
        printf '%s\n' "$preflight_out" | sed 's/^/    preflight| /'
    fi
else
    fatal "bin/cosmikase-preflight not found or not executable"
fi

echo
if [[ $fail -eq 0 ]]; then
    echo "=== container smoke assertions PASSED ==="
    exit 0
fi
echo "=== container smoke assertions FAILED ==="
exit 1
