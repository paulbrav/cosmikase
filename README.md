# Cosmikase - COSMIC Omakase for Pop!_OS

Pop!_OS 24 workstation configuration with COSMIC hotkeys, apt + Flatpak packages, Ghostty terminal, and a comprehensive theme system.

## Quick Start

```bash
# 1. Clone the repo
git clone https://github.com/paulbrav/omarchy-for-popos ~/Repos/cosmikase
cd ~/Repos/cosmikase

# 2. Install dependencies
make setup

# 3. Review configuration
nano cosmikase.yaml

# 4. Run the installer
make install

# 5. (Optional) Dry-run to preview changes
make dry-run
```

## Architecture

This project uses:
- **Ansible** for package installation and system configuration
- **chezmoi** for dotfile management with theme templating
- **YAML configuration** (`cosmikase.yaml`) to control what gets installed

## What Gets Installed

### Packages
- **Core CLI tools**: fzf, zoxide, ripgrep, fd, bat, btop, tmux, zellij, neovim, eza, gum
- **Build tools**: build-essential, cmake, ninja-build
- **GUI apps**: xournalpp, fonts-jetbrains-mono
- **Terminals**: Ghostty (built from source), Kitty, Alacritty

### Runtimes
- Rust (via rustup)
- Bun (JavaScript runtime)
- uv (Python package manager)
- nvm + Node.js LTS
- Julia

### Flatpak Apps
- Obsidian, LocalSend, Flatseal
- Bitwarden, ProtonVPN, Proton Mail
- Discord, Signal, Telegram
- Chromium, OnlyOffice

### AI Tools
- Cursor (AI code editor)
- Antigravity, Claude Code, OpenCode

### NPM Global Packages
- Bitwarden CLI (secrets management)
- OpenAI Codex CLI

## Cursor Extensions

Manage Cursor/VS Code extensions via a text file for easy reinstallation:

```bash
# Export your current extensions
cosmikase-cursor-extensions export

# Install extensions on a new machine
cosmikase-cursor-extensions install

# See what's different between list and installed
cosmikase-cursor-extensions diff
```

Edit `~/.config/Cursor/extensions.txt` to customize your extension list.

## Cursor Rules

### Sharing Rules Across Projects

**Recommended approach: Template Copy**

Keep a canonical set of rules in a dotfiles repo (e.g., [paulbrav/dotfiles](https://github.com/paulbrav/dotfiles)), then copy them to each project where they can diverge as needed:

```bash
# Copy rules to a new project from your dotfiles
cp -r ~/dotfiles/.cursor/rules .cursor/rules

# Or clone fresh from GitHub
git clone --depth 1 https://github.com/paulbrav/dotfiles /tmp/dotfiles
cp -r /tmp/dotfiles/.cursor/rules .cursor/rules
```

This approach:
- Gives each project its own copy that can be customized
- Tracks per-project changes in that project's git history
- Avoids symlink complexity and submodule overhead
- Lets you update the canonical version independently

### Rule File Format

Rules use `.mdc` (Markdown Cursor) format:

```markdown
---
description: Python development guidelines
globs: ["**/*.py"]
alwaysApply: false
---

# Python Rules

- Use type hints for all function signatures
- Prefer dataclasses over plain dicts for structured data
```

### Updating Rules

When you improve your canonical rules:

```bash
# See what changed
diff -r ~/dotfiles/.cursor/rules .cursor/rules

# Pull in updates (review before overwriting)
cp ~/dotfiles/.cursor/rules/python.mdc .cursor/rules/
```

## Theme System

15+ themes available, consistently applied across:
- Terminal emulators (Ghostty, Kitty, Alacritty)
- Development tools (Neovim, btop, Starship prompt)
- Desktop (COSMIC wallpaper, dark/light mode)

### Switching Themes

```bash
# CLI
cosmikase-theme tokyo-night

# Interactive TUI
theme-tui

# Interactive menu (gum)
cosmikase
```

### Available Themes

| Theme | Description |
|-------|-------------|
| `catppuccin` | Pastel dark theme |
| `catppuccin-latte` | Pastel light theme |
| `ethereal` | Dreamy ethereal palette |
| `everforest` | Forest green aesthetic |
| `flexoki-light` | Warm, paper-like light theme |
| `gruvbox` | Retro warm earth tones |
| `hackerman` | Matrix-inspired green |
| `kanagawa` | Japanese-inspired muted colors |
| `matte-black` | High contrast minimal |
| `nord` | Arctic-inspired cool palette |
| `osaka-jade` | Cyan and jade aesthetic |
| `pop-default` | Pop!_OS orange and teal |
| `ristretto` | Coffee-inspired warm theme |
| `rose-pine` | Soft rosé pastels |
| `tokyo-night` | Deep blues with vibrant accents |

## Shell Utilities

Cosmikase adds a handful of convenience functions/aliases via:

```bash
~/.config/shell/aliases/cosmikase_aliases.sh
```

Highlights:
- `compress <path>`: create `<path>.tar.gz`
- `decompress <archive.tar.gz>`: extract a tar.gz
- `webm2mp4 <input.webm>`: convert WebM recordings to MP4 (requires `ffmpeg`)
- `iso2sd <input.iso> </dev/sdX>`: write an ISO to a removable drive (destructive; prompts for confirmation)
- `dps`, `dlog <container>`, `dexec <container> [cmd]`: Docker helpers

## Configuration

Edit `cosmikase.yaml` to customize your installation:

```yaml
defaults:
  install: true
  ghostty: true
  theme: nord

apt:
  core:
    - name: fzf
      install: true
    - name: steam-installer
      install: false  # Opt-out

flatpak:
  utility:
    - id: md.obsidian.Obsidian
      install: true
```

## Development

```bash
# Install dev dependencies
make setup

# Run linters
make lint

# Format code
make fmt

# Run tests
make test

# Dry-run Ansible
make dry-run

# Interactive menu
make menu

# Build Exa plugin
make exa-build

# Install Exa plugin
make exa-install

# Clean Exa build
make exa-clean
```

## Interactive Menu (gum)

If `gum` is installed, you can use a single entrypoint to discover common actions:

```bash
cosmikase
```

Menu options include:
- Theme selection (launches `theme-tui` when available)
- Optional software installation (from `cosmikase.yaml`, installs items marked `install: false`)
- Docker development databases (PostgreSQL/MySQL/Redis/MongoDB)
- System update (`cosmikase-update`)
- Power settings (`cosmikase-power-helper`)
- Cursor extensions (`cosmikase-cursor-extensions`)

### Safety / Undo
- Optional software installs can be removed with `sudo apt remove <pkg>` or `flatpak uninstall <app-id>`.
- Databases are created as Docker containers named `cosmikase-*`. Remove them with:

```bash
docker rm -f cosmikase-postgres cosmikase-mysql cosmikase-redis cosmikase-mongodb
```

## Exa Launcher Plugin

A Pop!_OS launcher plugin for AI-powered web search via [Exa.ai](https://exa.ai/).

### Installation

```bash
# Build and install (requires Rust)
make exa-install
```

### Configuration

Set your Exa API key:

```bash
# Option 1: Environment variable
export EXA_API_KEY="your-api-key"

# Option 2: Config file (~/.config/exa-launcher/config.toml)
api_key = "your-api-key"
num_results = 8
```

### Usage

1. Open Pop Launcher with `Super` key
2. Type `exa ` followed by your search query
3. Press Enter to open a result in your browser

See [plugins/exa-launcher/README.md](plugins/exa-launcher/README.md) for details.

## Secrets Management with Bitwarden

API keys and secrets are managed via [chezmoi](https://chezmoi.io/) + [Bitwarden CLI](https://bitwarden.com/help/cli/), keeping sensitive data out of version control.

### Setup

1. **Store API keys in Bitwarden** as Login items (key in password field) or use custom fields
2. **Login and unlock Bitwarden CLI:**
   ```bash
   bw login
   export BW_SESSION="$(bw unlock --raw)"
   ```

3. **Edit the secrets template** to reference your Bitwarden items:
   ```bash
   chezmoi edit ~/.config/shell/secrets.sh
   ```
   
   Example template content:
   ```bash
   export EXA_API_KEY="{{ (bitwarden "item" "EXA API Key").login.password }}"
   export ANTHROPIC_API_KEY="{{ (bitwarden "item" "Anthropic API").login.password }}"
   ```

4. **Apply chezmoi** to generate the secrets file:
   ```bash
   chezmoi apply
   ```

### How It Works

- `secrets.sh.tmpl` is a chezmoi template that fetches secrets from Bitwarden at apply time
- The generated `~/.config/shell/secrets.sh` is sourced by your shell config
- Secrets are stored locally after `chezmoi apply`, not synced to git

### Re-syncing Secrets

After rotating keys in Bitwarden:
```bash
export BW_SESSION="$(bw unlock --raw)"
chezmoi apply
```

## Directory Structure

```
├── ansible/                 # Ansible playbook and roles
│   ├── playbook.yml        # Main entry point
│   └── roles/              # Modular installation roles
│       ├── packages/       # apt + flatpak
│       ├── runtimes/       # rust, bun, uv, nvm
│       ├── ghostty/        # build from source
│       ├── tools/          # AI tools, security
│       └── dotfiles/       # chezmoi apply
├── chezmoi/                # Dotfile source for chezmoi
│   ├── dot_config/         # ~/.config files (with .tmpl templates)
│   └── run_after_*.sh.tmpl # Post-apply scripts
├── bin/                    # Helper scripts
├── docs/                   # Documentation
├── plugins/                # Pop Launcher plugins
│   └── exa-launcher/       # Exa.ai search plugin
├── themes/                 # Theme definitions (15+ themes)
├── src/cosmikase/          # Python utilities (theme-tui)
└── cosmikase.yaml           # Main configuration
```

## Documentation

### Getting Started
- [CLI Reference](docs/cli-reference.md) - Complete command documentation
- [Configuration Reference](docs/configuration-reference.md) - Full `cosmikase.yaml` schema
- [Troubleshooting Guide](docs/troubleshooting.md) - Common issues and solutions

### Guides
- [Architecture Guide](docs/architecture.md) - System design and component interactions
- [Development Guide](docs/development.md) - Contributing and development setup
- [Editor Theming Guide](docs/editor-theming.md) - Cursor and Antigravity theming
- [COSMIC Theming Guide](docs/cosmic-theming.md) - Desktop environment theming
- [Zellij Guide](docs/zellij.md) - Terminal multiplexer configuration

### Manual Configuration
- [YubiKey Setup](docs/yubikey-setup.md) - PAM and SSH integration
- [Browser Sandboxing](docs/firejail-browsers.md) - Firejail configuration
- [Backup Strategy](docs/backup-strategy.md) - rsync and Timeshift setup
- [Interactive Menu](docs/cosmikase-menu.md) - Optional software and databases

### Theme System
- [Theme Documentation](themes/README.md) - Theme structure and usage
- [Wallpapers](themes/WALLPAPERS.md) - Wallpaper sources and management

## HP ZBook Ultra G1a Notes

Pop!_OS uses its own kernel. If webcam/suspend issues occur:
- Consider Ubuntu 24.04 OEM partition for `linux-oem-24.04b`
- Optional kernel params: `amd_iommu=off pcie_aspm=off`

## Testing

```bash
# Container smoke test (Ubuntu 24)
./tests/container-smoke.sh

# Full test in Pop!_OS VM
make install
```

## Uninstall

```bash
# Remove dotfile symlinks
chezmoi purge

# Remove shell integration
# Edit ~/.bashrc and remove the COSMIKASE MANAGED BLOCK
```

## License

MIT
