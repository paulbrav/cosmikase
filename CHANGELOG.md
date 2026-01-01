# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `.editorconfig` for consistent code formatting across editors
- `tests/conftest.py` with shared pytest fixtures
- Pydantic-based configuration schema validation (`src/cosmikase/schema.py`)
- `make validate` target for configuration validation
- `cosmikase-validate-config` CLI command
- Unified CLI entry point (`cosmikase-cli`) with subcommands
- Random password generation for database setup (with openssl fallback)

### Changed
- Renamed all `omarchy-pop-*` scripts and references to `cosmikase-*`
- Renamed shell library from `omarchy-pop-lib.sh` to `cosmikase-lib.sh`
- Renamed documentation file `omarchy-pop-menu.md` to `cosmikase-menu.md`
- Docker container names changed from `omarchy-*` to `cosmikase-*`
- Updated ASCII art banner in main menu to show "COSMIKASE"
- Improved security for database passwords (no longer uses weak defaults)
- Fixed Ansible idempotency in dotfiles role (`changed_when: false`)

### Fixed
- Test imports now use `cosmikase` module instead of `omarchy_pop`
- Removed stale `omarchy_pop.egg-info` directory

### Removed
- Deleted duplicate `bin/omarchy-pop-lib.sh` file
- Removed legacy `omarchy` naming throughout codebase

## [0.2.0] - 2025-01-01

### Added
- Theme TUI for interactive theme browsing
- Chezmoi integration for dotfile management
- COSMIC desktop theming support
- Multiple terminal emulator support (Ghostty, Kitty, Alacritty)
- Pop Launcher plugins (Exa, Bitwarden)
- Configuration-driven installation via `cosmikase.yaml`

### Changed
- Migrated from standalone scripts to Ansible-based installation
- Consolidated theme system with unified manifest format

## [0.1.0] - 2024-12-01

### Added
- Initial release
- Basic theme switching functionality
- APT and Flatpak package management
- Runtime installation (Rust, Node.js, Bun, Julia)

