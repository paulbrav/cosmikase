# Makefile for omarchy-pop
# Unified development commands for package installation, dotfile management, and linting

.PHONY: install setup update lint test theme clean fmt help dry-run exa-build exa-install exa-clean bw-build bw-install bw-clean

ANSIBLE_DIR := ansible
CHEZMOI_SOURCE := $(PWD)/chezmoi
CONFIG_FILE ?= omarchy-pop.yaml

# Default target
help:
	@echo "omarchy-pop - Dotfiles and system configuration for Pop!_OS"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Setup & Installation:"
	@echo "  setup       Install development dependencies (uv sync, chezmoi init)"
	@echo "  install     Run full Ansible playbook to configure system"
	@echo "  update      Update all installed packages and runtimes"
	@echo "  dry-run     Run Ansible in check mode (no changes)"
	@echo ""
	@echo "Dotfiles & Themes:"
	@echo "  theme       Apply current theme via chezmoi"
	@echo "  dotfiles    Apply all dotfiles via chezmoi"
	@echo ""
	@echo "Exa Launcher Plugin:"
	@echo "  exa-build   Build the exa-launcher plugin (requires Rust)"
	@echo "  exa-install Build and install plugin to ~/.local/share/pop-launcher/plugins/exa"
	@echo "  exa-clean   Clean exa-launcher build artifacts"
	@echo ""
	@echo "Bitwarden Launcher Plugin:"
	@echo "  bw-build    Build the bw-launcher plugin (requires Rust)"
	@echo "  bw-install  Build and install plugin to ~/.local/share/pop-launcher/plugins/bw"
	@echo "  bw-clean    Clean bw-launcher build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  lint        Run all linters (shellcheck, ruff, ansible-lint)"
	@echo "  fmt         Format Python code with ruff"
	@echo "  test        Run container smoke tests"
	@echo "  clean       Remove generated files and caches"

# Setup development environment
setup:
	@echo "==> Ensuring uv is installed..."
	@command -v uv >/dev/null 2>&1 || (echo "Installing uv..." && curl -LsSf https://astral.sh/uv/install.sh | sh)
	@echo "==> Installing Python dependencies (including ansible)..."
	$$HOME/.local/bin/uv sync --all-extras
	@echo "==> Installing Ansible collections..."
	$$HOME/.local/bin/uv run ansible-galaxy collection install community.general
	@echo "==> Ensuring chezmoi is installed..."
	@command -v chezmoi >/dev/null 2>&1 || (echo "Installing chezmoi..." && curl -sfL https://get.chezmoi.io | BINDIR=$$HOME/.local/bin sh)
	@echo "==> Initializing chezmoi..."
	$$HOME/.local/bin/chezmoi init --source=$(CHEZMOI_SOURCE) || true
	@echo "==> Setup complete!"

# Run full Ansible playbook
install:
	@echo "==> Running Ansible playbook..."
	cd $(ANSIBLE_DIR) && $$HOME/.local/bin/uv run ansible-playbook -i inventory.yml playbook.yml -K -e "config_file=$(realpath $(CONFIG_FILE))"

# Dry-run Ansible (check mode)
dry-run:
	@echo "==> Running Ansible in check mode (no changes)..."
	cd $(ANSIBLE_DIR) && $$HOME/.local/bin/uv run ansible-playbook -i inventory.yml playbook.yml --check -K -e "config_file=$(realpath $(CONFIG_FILE))"

# Update all installed packages and runtimes
update:
	@echo "==> Running system update..."
	./bin/omarchy-pop-update

# Apply dotfiles via chezmoi
dotfiles:
	chezmoi apply

# Apply theme (re-apply all theme-aware dotfiles)
theme:
	chezmoi apply

# Lint all code
lint: lint-shell lint-python lint-ansible

lint-shell:
	@echo "==> Running shellcheck on bin/*..."
	shellcheck bin/* || true

lint-python:
	@echo "==> Running ruff on src/..."
	$$HOME/.local/bin/uv run ruff check src/

lint-ansible:
	@echo "==> Running ansible-lint..."
	cd $(ANSIBLE_DIR) && $$HOME/.local/bin/uv run ansible-lint playbook.yml || true

# Format Python code
fmt:
	@echo "==> Formatting Python code..."
	$$HOME/.local/bin/uv run ruff format src/
	$$HOME/.local/bin/uv run ruff check --fix src/

# Run tests
test:
	@echo "==> Running container smoke test..."
	./tests/container-smoke.sh

# Clean generated files
clean:
	@echo "==> Cleaning up..."
	rm -rf .ruff_cache __pycache__ src/**/__pycache__
	rm -rf .venv
	@echo "==> Clean complete"

# =============================================================================
# Exa Launcher Plugin
# =============================================================================

EXA_PLUGIN_DIR := plugins/exa-launcher
EXA_INSTALL_DIR := $(HOME)/.local/share/pop-launcher/plugins/exa

# Build the exa-launcher plugin
exa-build:
	@echo "==> Building exa-launcher plugin..."
	cd $(EXA_PLUGIN_DIR) && cargo build --release
	@echo "==> Build complete: $(EXA_PLUGIN_DIR)/target/release/exa-launcher"

# Install the exa-launcher plugin to user's local plugin directory
exa-install: exa-build
	@echo "==> Installing exa-launcher plugin..."
	mkdir -p $(EXA_INSTALL_DIR)
	cp $(EXA_PLUGIN_DIR)/target/release/exa-launcher $(EXA_INSTALL_DIR)/
	cp $(EXA_PLUGIN_DIR)/plugin.ron $(EXA_INSTALL_DIR)/
	@echo "==> Plugin installed to $(EXA_INSTALL_DIR)"
	@echo ""
	@echo "NOTE: Set your Exa API key via environment variable:"
	@echo "  export EXA_API_KEY='your-api-key'"
	@echo ""
	@echo "Or create a config file at ~/.config/exa-launcher/config.toml:"
	@echo "  api_key = \"your-api-key\""
	@echo ""
	@echo "Usage: Open Pop Launcher (Super key) and type 'exa your search query'"

# Clean exa-launcher build artifacts
exa-clean:
	@echo "==> Cleaning exa-launcher build..."
	cd $(EXA_PLUGIN_DIR) && cargo clean

# =============================================================================
# Bitwarden Launcher Plugin
# =============================================================================

BW_PLUGIN_DIR := plugins/bw-launcher
BW_INSTALL_DIR := $(HOME)/.local/share/pop-launcher/plugins/bw

# Build the bw-launcher plugin
bw-build:
	@echo "==> Building bw-launcher plugin..."
	cd $(BW_PLUGIN_DIR) && cargo build --release
	@echo "==> Build complete: $(BW_PLUGIN_DIR)/target/release/bw-launcher"

# Install the bw-launcher plugin to user's local plugin directory
bw-install: bw-build
	@echo "==> Installing bw-launcher plugin..."
	mkdir -p $(BW_INSTALL_DIR)
	cp $(BW_PLUGIN_DIR)/target/release/bw-launcher $(BW_INSTALL_DIR)/
	cp $(BW_PLUGIN_DIR)/plugin.ron $(BW_INSTALL_DIR)/
	@echo "==> Plugin installed to $(BW_INSTALL_DIR)"
	@echo ""
	@echo "NOTE: Bitwarden CLI must be installed and logged in."
	@echo ""
	@echo "To store your session in the keyring:"
	@echo "  BW_SESSION=\$$(bw unlock --raw)"
	@echo "  echo -n \"\$$BW_SESSION\" | secret-tool store --label=\"Bitwarden Session\" session bw-launcher"
	@echo ""
	@echo "Usage: Open Pop Launcher (Super key) and type 'bw your search query'"

# Clean bw-launcher build artifacts
bw-clean:
	@echo "==> Cleaning bw-launcher build..."
	cd $(BW_PLUGIN_DIR) && cargo clean

