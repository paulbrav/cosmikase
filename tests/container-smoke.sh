#!/usr/bin/env bash
# Container smoke test for cosmikase
# Tests the Ansible playbook in a clean Ubuntu container
set -euo pipefail

ENGINE="${ENGINE:-podman}"
IMAGE="${IMAGE:-ubuntu:24.04}"
PREBUILT_IMAGE="${PREBUILT_IMAGE:-}"
REPO_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${CONFIG:-/workspace/tests/test-config.yaml}"
USE_PREBUILT="${USE_PREBUILT:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step() { echo -e "${CYAN}[STEP]${NC} $*"; }

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Run smoke test in a container (Ansible check mode).

Options:
    --engine ENGINE     Container engine to use (podman|docker) [default: podman]
    --image IMAGE       Base image to use [default: ubuntu:24.04]
    --prebuilt IMAGE    Use pre-built image (skips dependency install)
    --config FILE       Config file path inside container [default: /workspace/tests/test-config.yaml]
    -h, --help          Show this help message

Environment Variables:
    ENGINE              Same as --engine
    IMAGE               Same as --image
    PREBUILT_IMAGE      Same as --prebuilt
    CONFIG              Same as --config
    USE_PREBUILT        Set to 'true' to use cosmikase-smoke image

Examples:
    # Run with podman (default)
    ./tests/container-smoke.sh

    # Run with docker
    ENGINE=docker ./tests/container-smoke.sh

    # Use pre-built image (faster, requires 'make container-build' first)
    USE_PREBUILT=true ./tests/container-smoke.sh

    # Use custom base image
    ./tests/container-smoke.sh --image ubuntu:22.04
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --engine)
            ENGINE="$2"
            shift 2
            ;;
        --image)
            IMAGE="$2"
            shift 2
            ;;
        --prebuilt)
            PREBUILT_IMAGE="$2"
            USE_PREBUILT="true"
            shift 2
            ;;
        --config)
            CONFIG="$2"
            shift 2
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
    log_error "Container engine $ENGINE not found"
    log_info "Set ENGINE=docker or install podman"
    exit 1
fi

# Start timer
START_TIME=$(date +%s)

log_step "=== Smoke Test Starting ==="
log_info "Engine: $ENGINE"
log_info "Repository: $REPO_DIR"
log_info "Config: $CONFIG"

# Determine which image to use
if [[ "$USE_PREBUILT" == "true" ]]; then
    CONTAINER_IMAGE="${PREBUILT_IMAGE:-cosmikase-smoke}"
    log_info "Using pre-built image: $CONTAINER_IMAGE"
    
    # Check if image exists
    if ! $ENGINE image exists "$CONTAINER_IMAGE" 2>/dev/null; then
        log_warn "Pre-built image not found, building..."
        $ENGINE build -f "$REPO_DIR/tests/Containerfile.smoke" -t "$CONTAINER_IMAGE" "$REPO_DIR"
    fi
    
    # Run with pre-built image (simpler command)
    log_step "Running smoke test with pre-built image..."
    
    $ENGINE run --rm -t \
        --privileged \
        -v "$REPO_DIR":/workspace:ro \
        -e "CONFIG_FILE=$CONFIG" \
        "$CONTAINER_IMAGE"
    
    EXIT_CODE=$?
else
    log_info "Using base image: $IMAGE"
    
    # Build test script to run inside container
    TEST_SCRIPT=$(cat << 'ENDSCRIPT'
#!/bin/bash
set -euo pipefail

echo "=== Installing base dependencies ==="
apt-get update
apt-get install -y sudo curl rsync unzip python3 python3-pip git software-properties-common

echo "=== Installing Ansible ==="
pip3 install --break-system-packages ansible

echo "=== Installing Ansible collections ==="
ansible-galaxy collection install community.general ansible.posix

echo "=== Running Ansible playbook (check mode) ==="
cd /workspace/ansible
ansible-playbook -i inventory.yml playbook.yml --check \
    -e "config_file=$CONFIG_FILE" \
    --skip-tags ghostty,dotfiles \
    -v

echo "=== Verifying Python module ==="
cd /workspace
pip3 install --break-system-packages pyyaml
python3 -c "from src.cosmikase.config import load_config, get_value; print('Python module OK')"

echo "=== Smoke test passed! ==="
ENDSCRIPT
)

    # Run the test in container
    log_step "Running smoke test in fresh container..."

    $ENGINE run --rm -t \
        --privileged \
        -v "$REPO_DIR":/workspace:ro \
        -e "CONFIG_FILE=$CONFIG" \
        "$IMAGE" \
        bash -c "$TEST_SCRIPT"
    
    EXIT_CODE=$?
fi

# Calculate duration
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

if [[ $EXIT_CODE -eq 0 ]]; then
    log_step "=== Test Complete ==="
    log_info "Duration: ${DURATION}s"
    log_info "Smoke test PASSED!"
else
    log_error "Smoke test failed with exit code $EXIT_CODE"
    log_info "Duration: ${DURATION}s"
fi

exit $EXIT_CODE
