#!/usr/bin/env bash
# container-full.sh - Full end-to-end container test for cosmikase
# Builds and runs a container that performs actual package installation
set -euo pipefail

ENGINE="${ENGINE:-podman}"
IMAGE_NAME="${IMAGE_NAME:-cosmikase-full}"
REPO_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${CONFIG:-/workspace/tests/test-config.yaml}"
NO_CACHE="${NO_CACHE:-false}"
SKIP_BUILD="${SKIP_BUILD:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step() { echo -e "${CYAN}[STEP]${NC} $*"; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Run full end-to-end install test in a container.

Options:
    --engine ENGINE     Container engine to use (podman|docker) [default: podman]
    --config FILE       Config file path inside container [default: /workspace/tests/test-config.yaml]
    --no-cache          Build without cache
    --skip-build        Skip building, use existing image
    -h, --help          Show this help message

Environment Variables:
    ENGINE              Same as --engine
    CONFIG              Same as --config
    NO_CACHE            Same as --no-cache (set to 'true')
    SKIP_BUILD          Same as --skip-build (set to 'true')

Examples:
    # Run with podman (default)
    ./tests/container-full.sh

    # Run with docker
    ENGINE=docker ./tests/container-full.sh

    # Rebuild without cache
    ./tests/container-full.sh --no-cache

    # Use existing image
    ./tests/container-full.sh --skip-build
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --engine)
            ENGINE="$2"
            shift 2
            ;;
        --config)
            CONFIG="$2"
            shift 2
            ;;
        --no-cache)
            NO_CACHE="true"
            shift
            ;;
        --skip-build)
            SKIP_BUILD="true"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Check container engine
if ! command -v "$ENGINE" >/dev/null 2>&1; then
    log_error "Container engine '$ENGINE' not found"
    log_info "Set ENGINE=docker or install podman"
    exit 1
fi

log_step "=== Full Install Container Test ==="
log_info "Engine: $ENGINE"
log_info "Repository: $REPO_DIR"
log_info "Config: $CONFIG"

# Start timer
START_TIME=$(date +%s)

# Build the container image
if [[ "$SKIP_BUILD" != "true" ]]; then
    log_step "Building container image..."
    
    BUILD_ARGS=(-f "$REPO_DIR/tests/Containerfile.full" -t "$IMAGE_NAME")
    
    if [[ "$NO_CACHE" == "true" ]]; then
        BUILD_ARGS+=(--no-cache)
    fi
    
    BUILD_ARGS+=("$REPO_DIR")
    
    if ! $ENGINE build "${BUILD_ARGS[@]}"; then
        log_error "Failed to build container image"
        exit 1
    fi
    
    log_info "Container image built: $IMAGE_NAME"
else
    log_info "Skipping build, using existing image: $IMAGE_NAME"
fi

# Run the container
log_step "Running full install test..."

RUN_ARGS=(
    run
    --rm
    -t
    --privileged
    -v "$REPO_DIR":/workspace
    -e "CONFIG_FILE=$CONFIG"
    "$IMAGE_NAME"
)

if ! $ENGINE "${RUN_ARGS[@]}"; then
    EXIT_CODE=$?
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    
    log_error "Full install test failed with exit code $EXIT_CODE"
    log_info "Duration: ${DURATION}s"
    exit $EXIT_CODE
fi

# Success
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

log_step "=== Test Complete ==="
log_info "Duration: ${DURATION}s"
log_info "Full install test PASSED!"

exit 0

