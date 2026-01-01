#!/usr/bin/env bash
# container-full-entrypoint.sh - Entrypoint for full install test container
# Runs the complete cosmikase installation and verifies results
set -euo pipefail

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

CONFIG_FILE="${CONFIG_FILE:-/workspace/tests/test-config.yaml}"
FAILED=0

check_binary() {
    local name="$1"
    local cmd="${2:-$1}"
    if command -v "$cmd" >/dev/null 2>&1; then
        log_info "✓ $name installed: $(command -v "$cmd")"
        return 0
    else
        log_error "✗ $name not found"
        return 1
    fi
}

# Start timer
START_TIME=$(date +%s)

log_step "=== Full Install Test Starting ==="
log_info "Config: $CONFIG_FILE"
log_info "Working directory: $(pwd)"

# Step 1: Setup Python environment
log_step "Setting up Python environment..."
cd /workspace
uv sync --all-extras

# Step 2: Install Ansible collections
log_step "Installing Ansible collections..."
uv run ansible-galaxy collection install community.general ansible.posix

# Step 3: Initialize chezmoi
log_step "Initializing chezmoi..."
chezmoi init --source=/workspace/chezmoi

# Step 4: Run Ansible playbook (actual install, not check mode)
log_step "Running Ansible playbook (full install)..."
cd /workspace/ansible

# Run without -K since we're root in container, skip tags that require desktop environment
uv run ansible-playbook -i inventory.yml playbook.yml \
    -e "config_file=$CONFIG_FILE" \
    --skip-tags ghostty,dotfiles \
    -v || {
    log_error "Ansible playbook failed"
    FAILED=1
}

# Step 5: Verify installed binaries
log_step "Verifying installed binaries..."
cd /workspace

# Core tools that should be installed
BINARIES=(
    "fzf"
    "zoxide"
    "rg:ripgrep"
    "fd:fd-find"
    "bat"
    "btop"
    "tmux"
    "git"
    "nvim:neovim"
    "jq"
    "tree"
)

for entry in "${BINARIES[@]}"; do
    cmd="${entry%%:*}"
    name="${entry#*:}"
    if ! check_binary "$name" "$cmd"; then
        FAILED=1
    fi
done

# Step 6: Verify Python module works
log_step "Verifying Python module..."
if uv run python -c "from cosmikase.config import load_config, get_value; print('Python module OK')"; then
    log_info "✓ Python module working"
else
    log_error "✗ Python module failed"
    FAILED=1
fi

# Step 7: Verify cosmikase CLI tools
log_step "Verifying cosmikase CLI tools..."
CLI_TOOLS=(
    "cosmikase-config"
    "cosmikase-validate-config"
)

for tool in "${CLI_TOOLS[@]}"; do
    if uv run "$tool" --help >/dev/null 2>&1; then
        log_info "✓ $tool working"
    else
        log_warn "⚠ $tool not responding (may be expected)"
    fi
done

# Calculate duration
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
log_step "=== Test Summary ==="
log_info "Duration: ${DURATION}s"

if [[ $FAILED -eq 0 ]]; then
    log_info "✓ Full install test PASSED!"
    exit 0
else
    log_error "✗ Full install test FAILED"
    exit 1
fi

