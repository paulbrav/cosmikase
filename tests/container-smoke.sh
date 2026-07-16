#!/usr/bin/env bash
# container-smoke.sh - build and run the cosmikase "build a machine" smoke test.
#
# Auto-detects docker or podman, builds tests/Containerfile.smoke (which runs
# COSMIKASE_CI=1 ./install.sh on a clean Ubuntu 24.04 as a non-root user), then
# runs the image so tests/container-smoke-assert.sh verifies the result.
#
# If no usable container engine is available, prints SKIPPED and exits 0 so it
# never hard-fails a machine that simply has no daemon.
set -euo pipefail

REPO_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_TAG="${IMAGE_TAG:-cosmikase-smoke:latest}"
CONTAINERFILE="$REPO_DIR/tests/Containerfile.smoke"
ENGINE="${ENGINE:-}"
KEEP_IMAGE="${KEEP_IMAGE:-false}"

# Colors (only when stdout is a TTY).
if [[ -t 1 ]]; then
    RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[1;33m'; CYAN=$'\033[0;36m'; NC=$'\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; CYAN=''; NC=''
fi
log_info()  { echo "${GREEN}[INFO]${NC} $*"; }
log_error() { echo "${RED}[ERROR]${NC} $*" >&2; }
log_step()  { echo "${CYAN}[STEP]${NC} $*"; }
log_skip()  { echo "${YELLOW}[SKIP]${NC} $*"; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Build and run the container smoke test (COSMIKASE_CI=1 ./install.sh + asserts).

Options:
    --engine ENGINE   Container engine: docker or podman (default: auto-detect)
    --keep            Do not remove the built image afterwards
    -h, --help        Show this help

Environment:
    ENGINE            Same as --engine
    IMAGE_TAG         Image tag to build (default: cosmikase-smoke:latest)
    KEEP_IMAGE        Set to 'true' to keep the image (same as --keep)
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --engine) ENGINE="$2"; shift 2 ;;
        --keep)   KEEP_IMAGE="true"; shift ;;
        -h|--help) usage; exit 0 ;;
        *) log_error "Unknown option: $1"; usage; exit 1 ;;
    esac
done

# Return 0 if the given engine exists and its daemon is usable.
engine_usable() {
    local eng="$1"
    command -v "$eng" >/dev/null 2>&1 || return 1
    "$eng" info >/dev/null 2>&1
}

# Auto-detect an engine if one was not forced.
if [[ -z "$ENGINE" ]]; then
    for candidate in docker podman; do
        if engine_usable "$candidate"; then
            ENGINE="$candidate"
            break
        fi
    done
fi

if [[ -z "$ENGINE" ]]; then
    log_skip "No usable container engine (docker/podman) found - skipping smoke test."
    exit 0
fi

if ! engine_usable "$ENGINE"; then
    log_skip "Container engine '$ENGINE' present but daemon unusable - skipping."
    exit 0
fi

START_TIME=$(date +%s)
log_step "=== cosmikase container smoke test ==="
log_info "Engine:      $ENGINE"
log_info "Repository:  $REPO_DIR"
log_info "Image tag:   $IMAGE_TAG"

log_step "Building image (runs COSMIKASE_CI=1 ./install.sh)..."
if ! "$ENGINE" build -f "$CONTAINERFILE" -t "$IMAGE_TAG" "$REPO_DIR"; then
    log_error "Image build failed (install.sh did not complete cleanly)."
    exit 1
fi

log_step "Running assertions in the built image..."
run_rc=0
"$ENGINE" run --rm "$IMAGE_TAG" || run_rc=$?

if [[ "$KEEP_IMAGE" != "true" ]]; then
    "$ENGINE" rmi -f "$IMAGE_TAG" >/dev/null 2>&1 || true
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

if [[ $run_rc -eq 0 ]]; then
    log_step "=== smoke test PASSED (${DURATION}s) ==="
else
    log_error "smoke test FAILED with exit code $run_rc (${DURATION}s)"
fi
exit "$run_rc"
