# Makefile — cosmikase developer conveniences.
# The real installer is ./install.sh (chezmoi is the single orchestrator).

CHEZMOI_SOURCE := $(CURDIR)/chezmoi
PLUGINS := exa bw ssh man clip
PLUGIN_ROOT := $(HOME)/.local/share/pop-launcher/plugins

.PHONY: help setup install apply preflight theme lint test plugins plugins-install clean

help:
	@echo "cosmikase — targets:"
	@echo "  setup            Install chezmoi and initialise the source dir (no apply)"
	@echo "  install          Full bootstrap via ./install.sh"
	@echo "  apply            chezmoi apply (re-apply dotfiles + changed package scripts)"
	@echo "  preflight        Run the hardware/COSMIC readiness check"
	@echo "  theme            Open the theme picker (bin/cosmikase-theme)"
	@echo "  lint             shellcheck shell scripts + ruff the python single-file scripts"
	@echo "  test             Run the pytest suite (tests/)"
	@echo "  plugins          Build the pop-launcher plugin workspace (release)"
	@echo "  plugins-install  Build + install all plugins into ~/.local/share/pop-launcher"
	@echo "  clean            Remove caches and build artifacts"

setup:
	@command -v chezmoi >/dev/null 2>&1 || { echo "Installing chezmoi…"; \
	  sh -c "$$(curl -fsLS get.chezmoi.io)" -- -b "$$HOME/.local/bin"; }
	chezmoi init --source=$(CHEZMOI_SOURCE)
	@echo "Ready. Run 'make install' to apply, or 'make apply' for dotfiles only."

install:
	./install.sh

apply:
	chezmoi apply

preflight:
	./bin/cosmikase-preflight

theme:
	./bin/cosmikase-theme

lint:
	./scripts/lint.sh
	@if command -v ruff >/dev/null 2>&1; then \
	  ruff check .; \
	else echo "ruff not installed; skipping python lint"; fi

test:
	uv run --with pytest,tomlkit,pyyaml pytest tests/ -x -q

plugins:
	cargo build --release --manifest-path plugins/Cargo.toml

plugins-install: plugins
	@for p in $(PLUGINS); do \
	  dest="$(PLUGIN_ROOT)/$$p"; \
	  mkdir -p "$$dest"; \
	  cp "plugins/target/release/$$p-launcher" "$$dest/"; \
	  cp "plugins/$$p-launcher/plugin.ron" "$$dest/"; \
	  echo "installed $$p -> $$dest"; \
	done

clean:
	rm -rf .ruff_cache .pytest_cache __pycache__
	cargo clean --manifest-path plugins/Cargo.toml 2>/dev/null || true
