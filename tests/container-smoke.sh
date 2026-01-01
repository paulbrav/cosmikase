#!/usr/bin/env bash
# Container smoke test for omarchy-pop
# Tests the Ansible playbook in a clean Ubuntu container
set -euo pipefail

ENGINE="${ENGINE:-podman}"
IMAGE="${IMAGE:-ubuntu:24.04}"
REPO_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG="${CONFIG:-/workspace/tests/test-config.yaml}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Check container engine
if ! command -v "$ENGINE" >/dev/null 2>&1; then
    log_error "Container engine $ENGINE not found"
    log_info "Set ENGINE=docker or install podman"
    exit 1
fi

log_info "Starting smoke test with $ENGINE using $IMAGE"
log_info "Repository: $REPO_DIR"
log_info "Config: $CONFIG"

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
python3 -c "from src.omarchy_pop.config import load_config, get_value; print('Python module OK')"

echo "=== Smoke test passed! ==="
ENDSCRIPT
)

# Run the test in container
log_info "Running test in container..."

$ENGINE run --rm -t \
    --privileged \
    -v "$REPO_DIR":/workspace:ro \
    -e "CONFIG_FILE=$CONFIG" \
    "$IMAGE" \
    bash -c "$TEST_SCRIPT"

EXIT_CODE=$?

if [[ $EXIT_CODE -eq 0 ]]; then
    log_info "Smoke test finished successfully!"
else
    log_error "Smoke test failed with exit code $EXIT_CODE"
fi

exit $EXIT_CODE
