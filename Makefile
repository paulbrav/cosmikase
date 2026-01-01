# Makefile for cosmikase
# Unified development commands for package installation, dotfile management, and linting

.PHONY: install setup update lint test test-cov test-smoke test-full test-container container-build theme clean fmt help dry-run validate exa-build exa-install exa-clean bw-build bw-install bw-clean menu

ANSIBLE_DIR := ansible
CHEZMOI_SOURCE := $(PWD)/chezmoi
CONFIG_FILE ?= cosmikase.yaml

# Tool paths - use PATH lookup with fallback to common install locations
UV := $(shell command -v uv 2>/dev/null || echo "$$HOME/.local/bin/uv")
CHEZMOI := $(shell command -v chezmoi 2>/dev/null || echo "$$HOME/.local/bin/chezmoi")

# Default target
help:
	@echo "cosmikase - Dotfiles and system configuration for Pop!_OS"
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "Setup & Installation:"
	@echo "  setup           Install development dependencies (uv sync, chezmoi init)"
	@echo "  install         Run full installation with beautiful gum UI ⛩️"
	@echo "  install-verbose Run Ansible directly (for debugging)"
	@echo "  update          Update all installed packages and runtimes"
	@echo "  dry-run         Run Ansible in check mode (no changes)"
	@echo ""
	@echo "Dotfiles & Themes:"
	@echo "  theme       Apply current theme via chezmoi"
	@echo "  dotfiles    Apply all dotfiles via chezmoi"
	@echo "  menu        Open the interactive cosmikase menu (requires gum)"
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
	@echo "  lint            Run all linters (shellcheck, ruff, ansible-lint)"
	@echo "  fmt             Format Python code with ruff"
	@echo "  test            Run pytest + container smoke test"
	@echo "  test-smoke      Fast container smoke test (check mode, ~2 min)"
	@echo "  test-full       Full container install test (~10 min)"
	@echo "  test-container  Run both container test tiers"
	@echo "  container-build Build test container images"
	@echo "  validate        Validate cosmikase.yaml configuration"
	@echo "  clean           Remove generated files and caches"

# Setup development environment
setup:
	@echo "==> Ensuring uv is installed..."
	@if ! command -v uv >/dev/null 2>&1; then \
		echo "Installing uv..."; \
		curl -LsSf https://astral.sh/uv/install.sh -o /tmp/uv-install.sh && sh /tmp/uv-install.sh && rm /tmp/uv-install.sh; \
	fi
	@echo "==> Installing Python dependencies (including ansible)..."
	$(UV) sync --all-extras
	@echo "==> Installing Ansible collections..."
	$(UV) run ansible-galaxy collection install community.general
	@echo "==> Ensuring chezmoi is installed..."
	@if ! command -v chezmoi >/dev/null 2>&1; then \
		echo "Installing chezmoi..."; \
		curl -sfL https://get.chezmoi.io -o /tmp/chezmoi-install.sh && BINDIR=$$HOME/.local/bin sh /tmp/chezmoi-install.sh && rm /tmp/chezmoi-install.sh; \
	fi
	@echo "==> Initializing chezmoi..."
	$(CHEZMOI) init --source=$(CHEZMOI_SOURCE)
	@echo "==> Setup complete!"

# Run full installation with beautiful gum UI
install:
	@./bin/cosmikase-install

# Run Ansible directly (verbose, for debugging)
install-verbose:
	@echo ""
	@echo "╔═══════════════════════════════════════════════════════════════════╗"
	@echo "║              ⛩️  COSMIKASE SYSTEM INSTALLATION                     ║"
	@echo "╠═══════════════════════════════════════════════════════════════════╣"
	@echo "║  This will install packages, runtimes, and configure your system  ║"
	@echo "║  Task timing is shown - longer tasks will display elapsed time    ║"
	@echo "╚═══════════════════════════════════════════════════════════════════╝"
	@echo ""
	cd $(ANSIBLE_DIR) && $(UV) run ansible-playbook -i inventory.yml playbook.yml -K -e "config_file=$(realpath $(CONFIG_FILE))"

# Dry-run Ansible (check mode)
dry-run:
	@echo "==> Running Ansible in check mode (no changes)..."
	cd $(ANSIBLE_DIR) && $(UV) run ansible-playbook -i inventory.yml playbook.yml --check -K -e "config_file=$(realpath $(CONFIG_FILE))"

# Update all installed packages and runtimes
update:
	@echo "==> Running system update..."
	./bin/cosmikase-update

# Apply dotfiles via chezmoi
dotfiles:
	chezmoi apply

# Apply theme (re-apply all theme-aware dotfiles)
theme:
	chezmoi apply

# Main interactive menu
menu:
	./bin/cosmikase

# Lint all code
lint: lint-shell lint-python lint-ansible

lint-shell:
	@echo "==> Running shellcheck on bin/*..."
	shellcheck bin/*

lint-python:
	@echo "==> Running ruff on src/..."
	$(UV) run ruff check src/

lint-ansible:
	@echo "==> Running ansible-lint..."
	cd $(ANSIBLE_DIR) && $(UV) run ansible-lint playbook.yml

# Format Python code
fmt:
	@echo "==> Formatting Python code..."
	$(UV) run ruff format src/
	$(UV) run ruff check --fix src/

# Run tests
test:
	@echo "==> Running Python unit tests..."
	$(UV) run pytest tests/ -v
	@echo "==> Running container smoke test..."
	./tests/container-smoke.sh

# Run tests with coverage
test-cov:
	@echo "==> Running Python unit tests with coverage..."
	$(UV) run pytest tests/ -v --cov=src/cosmikase --cov-report=term-missing --cov-report=html
	@echo "==> Coverage report: htmlcov/index.html"

# =============================================================================
# Container Testing
# =============================================================================

# Container engine - podman by default, override with ENGINE=docker
ENGINE ?= podman

# Fast container smoke test (Ansible check mode, ~2 min)
test-smoke:
	@echo "==> Running container smoke test..."
	ENGINE=$(ENGINE) ./tests/container-smoke.sh

# Full container install test (~10 min)
test-full:
	@echo "==> Running full container install test..."
	ENGINE=$(ENGINE) ./tests/container-full.sh

# Run both container test tiers
test-container: test-smoke test-full
	@echo "==> All container tests complete!"

# Build test container images (for caching)
container-build:
	@echo "==> Building smoke test container..."
	$(ENGINE) build -f tests/Containerfile.smoke -t cosmikase-smoke .
	@echo "==> Building full test container..."
	$(ENGINE) build -f tests/Containerfile.full -t cosmikase-full .
	@echo "==> Container images built!"

# Validate configuration
validate:
	@echo "==> Validating cosmikase.yaml..."
	$(UV) run cosmikase-validate-config $(CONFIG_FILE)

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

