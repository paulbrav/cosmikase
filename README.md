# Omarchy-style Pop!_OS Setup

Pop!_OS 24 workstation configuration with COSMIC hotkeys, apt + Flatpak packages, Ghostty terminal, and a comprehensive theme system.

## Quick Start

```bash
# 1. Clone the repo
git clone https://github.com/paulbrav/omarchy-for-popos ~/Repos/omarchy-for-popos
cd ~/Repos/omarchy-for-popos

# 2. Install dependencies
make setup

# 3. Review configuration
nano omarchy-pop.yaml

# 4. Run the installer
make install

# 5. (Optional) Dry-run to preview changes
make dry-run
```

## Architecture

This project uses:
- **Ansible** for package installation and system configuration
- **chezmoi** for dotfile management with theme templating
- **YAML configuration** (`omarchy-pop.yaml`) to control what gets installed

## What Gets Installed

### Packages
- **Core CLI tools**: fzf, zoxide, ripgrep, fd, bat, btop, tmux, neovim, eza
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
omarchy-cursor-extensions export

# Install extensions on a new machine
omarchy-cursor-extensions install

# See what's different between list and installed
omarchy-cursor-extensions diff
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
omarchy-pop-theme tokyo-night

# Interactive TUI
theme-tui
```

### Available Themes

| Theme | Description |
|-------|-------------|
| `tokyo-night` | Deep blues with vibrant accents |
| `nord` | Arctic-inspired cool palette |
| `gruvbox` | Retro warm earth tones |
| `catppuccin` | Pastel dark theme |
| `catppuccin-latte` | Pastel light theme |
| `rose-pine` | Soft rosé pastels |
| `kanagawa` | Japanese-inspired muted colors |
| `everforest` | Forest green aesthetic |
| `hackerman` | Matrix-inspired green |
| `matte-black` | High contrast minimal |
| `osaka-jade` | Cyan and jade aesthetic |
| `pop-default` | Pop!_OS orange and teal |

## Configuration

Edit `omarchy-pop.yaml` to customize your installation:

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

# Build Exa plugin
make exa-build

# Install Exa plugin
make exa-install

# Clean Exa build
make exa-clean
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
├── src/omarchy_pop/        # Python utilities (theme-tui)
└── omarchy-pop.yaml        # Main configuration
```

## Manual Steps

Some features require manual configuration:

### YubiKey Setup
See [docs/yubikey-setup.md](docs/yubikey-setup.md) for PAM and SSH integration.

### Browser Sandboxing
See [docs/firejail-browsers.md](docs/firejail-browsers.md) for Firejail configuration.

### Backups
See [docs/backup-strategy.md](docs/backup-strategy.md) for rsync and Timeshift setup.

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
# Edit ~/.bashrc and remove the OMARCHY-POP MANAGED BLOCK
```

## License

MIT
