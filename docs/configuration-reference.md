# Configuration Reference

Complete reference for the `omarchy-pop.yaml` configuration file schema.

## Table of Contents

- [Overview](#overview)
- [Top-Level Sections](#top-level-sections)
  - [defaults](#defaults)
  - [apt](#apt)
  - [flatpak](#flatpak)
  - [fonts](#fonts)
  - [installers](#installers)
  - [npm](#npm)
  - [scripts](#scripts)
  - [uv_tools](#uv_tools)
  - [themes](#themes)
  - [hp_zbook_ultra](#hp_zbook_ultra)
- [Item Schema](#item-schema)
- [Validation Rules](#validation-rules)
- [Examples](#examples)

---

## Overview

The `omarchy-pop.yaml` file controls what gets installed and configured by the Ansible playbook. It uses a simple YAML structure with sections for different package types and installation methods.

**File Location:**
- Default: `omarchy-pop.yaml` in repository root
- Can be overridden with `--config` flag or `OMARCHY_POP_CONFIG` environment variable

**Basic Structure:**
```yaml
defaults:
  install: true
  theme: nord

apt:
  core:
    - name: package-name
      install: true

flatpak:
  utility:
    - id: app.id
      install: true
```

---

## Top-Level Sections

### defaults

Global default settings applied across the configuration.

**Schema:**
```yaml
defaults:
  install: boolean          # Default install flag (default: true)
  ghostty: boolean         # Build Ghostty from source (default: true)
  yubikey_setup: boolean   # Enable YubiKey setup (default: false)
  theme: string           # Default theme name (default: nord)
  run_fw_update: boolean  # Run firmware updates (default: true)
  run_recovery_upgrade: boolean  # Run recovery upgrade (default: false)
```

**Fields:**
- `install`: If `false`, all items default to `install: false` unless explicitly set to `true`
- `ghostty`: Whether to build Ghostty terminal from source (requires Zig 0.13+)
- `yubikey_setup`: Enable YubiKey PAM/SSH setup (see [yubikey-setup.md](yubikey-setup.md))
- `theme`: Default theme to apply (must match a theme in `themes/` directory)
- `run_fw_update`: Run `fwupdmgr` to check for firmware updates
- `run_recovery_upgrade`: Run `pop-upgrade recovery upgrade` (use with caution)

**Example:**
```yaml
defaults:
  install: true
  ghostty: true
  theme: tokyo-night
  yubikey_setup: false
```

---

### apt

APT package installation configuration. Organized into groups for logical organization.

**Schema:**
```yaml
apt:
  <group-name>:
    - name: string          # Package name (required)
      desc: string          # Description (optional)
      alias: string         # Alternative command name (optional)
      install: boolean     # Install this package (default: true)
```

**Groups:**
- `core`: Essential CLI tools (fzf, zoxide, ripgrep, etc.)
- `yubikey`: YubiKey-related packages (yubikey-manager, libpam-u2f, etc.)
- `gui`: GUI applications (xournalpp, fonts)
- `terminal`: Terminal emulators and related tools (ghostty, kitty, alacritty, docker, etc.)

**Special Fields:**
- `alias`: Use when package name differs from command name (e.g., `fd-find` → `fdfind`)
- `source`: Special value `"source"` indicates build from source (used for Ghostty)
- `note`: Additional notes about the package

**Example:**
```yaml
apt:
  core:
    - name: fzf
      desc: General-purpose command-line fuzzy finder
      install: true
    - name: fd-find
      desc: Simple, fast alternative to find
      alias: fdfind
      install: true
  terminal:
    - name: ghostty
      desc: Fast terminal emulator
      source: source
      note: "Compiling from source (requires Zig 0.13+)"
      install: true
```

**Querying:**
```bash
# List enabled packages in a group
omarchy-config list apt core

# List only names
omarchy-config list apt core --names-only

# List disabled packages
omarchy-config list apt core --disabled
```

---

### flatpak

Flatpak application installation configuration.

**Schema:**
```yaml
flatpak:
  <group-name>:
    - id: string           # Flatpak application ID (required)
      desc: string         # Description (optional)
      install: boolean     # Install this app (default: true)
```

**Groups:**
- `utility`: Utility applications (Obsidian, LocalSend, Flatseal, etc.)
- `productivity`: Productivity tools (OnlyOffice, Standard Notes, etc.)
- `communication`: Communication apps (Discord, Signal, Telegram, etc.)

**Application IDs:**
Flatpak IDs use reverse DNS notation:
- `md.obsidian.Obsidian`
- `com.system76.KeyboardConfigurator`
- `org.signal.Signal`

**Example:**
```yaml
flatpak:
  utility:
    - id: md.obsidian.Obsidian
      desc: A knowledge base that works on top of a local folder
      install: true
    - id: com.spotify.Client
      desc: Music streaming service
      install: false  # Optional - can install via omarchy-pop-install
```

**Querying:**
```bash
# List enabled Flatpak apps
omarchy-config list flatpak utility

# List as JSON
omarchy-config list flatpak utility --json
```

---

### fonts

Font installation configuration.

**Schema:**
```yaml
fonts:
  <group-name>:
    - name: string         # Font name (required)
      desc: string         # Description (optional)
      url: string          # Download URL (required)
      install: boolean     # Install this font (default: true)
```

**Groups:**
- `nerd`: Nerd Fonts (icon fonts for terminals)

**Example:**
```yaml
fonts:
  nerd:
    - name: JetBrainsMono Nerd Font
      desc: Developer font with icons
      url: https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip
      install: true
```

---

### installers

Custom installer configuration for runtimes and tools that don't use standard package managers.

**Schema:**
```yaml
installers:
  <group-name>:
    - name: string         # Tool name (required)
      desc: string         # Description (optional)
      method: string       # Installation method (required)
      check: string        # Command to check if installed (required)
      install: boolean     # Install this tool (default: true)
      # Method-specific fields (see below)
```

**Groups:**
- `runtimes`: Programming language runtimes (Rust, Bun, Node.js, Julia, etc.)
- `ai_tools`: AI coding assistants (Cursor, Antigravity, Codex, etc.)
- `security`: Security tools (Brave browser, Dangerzone, etc.)

**Installation Methods:**

#### `script`
Run an installation script from a URL.

```yaml
- name: starship
  method: script
  url: https://starship.rs/install.sh
  args: ""  # Optional arguments to pass to script
  check: starship
  install: true
```

#### `custom_*`
Custom installation handlers defined in Ansible roles.

```yaml
- name: antigravity
  method: custom_antigravity
  check: antigravity
  install: true

- name: brave-browser
  method: custom_brave
  check: brave-browser
  install: true
```

#### `deb`
Download and install a `.deb` package.

```yaml
- name: cursor
  method: deb
  deb_url: https://api2.cursor.sh/updates/download/golden/linux-x64-deb/cursor/latest
  check: cursor
  install: true
```

#### `npm`
Install via npm (global).

```yaml
- name: codex
  method: npm
  npm_package: "@openai/codex"
  check: codex
  install: true
```

#### `bun`
Install via Bun (global).

```yaml
- name: opencode
  method: bun
  bun_package: opencode-ai
  check: opencode
  install: true
```

#### `manual`
Manual installation (no automation).

```yaml
- name: amp
  method: manual
  note: "Check https://ampcode.com for install instructions"
  install: false
```

**Example:**
```yaml
installers:
  runtimes:
    - name: rust
      desc: Systems programming language
      method: script
      url: https://sh.rustup.rs
      args: -y
      check: rustc
      install: true
  ai_tools:
    - name: cursor
      desc: The AI Code Editor
      method: deb
      deb_url: "https://api2.cursor.sh/updates/download/golden/linux-x64-deb/cursor/latest"
      check: cursor
      install: true
```

---

### npm

Global NPM package installation.

**Schema:**
```yaml
npm:
  - name: string          # Package name (required, can include scope like @scope/pkg)
    desc: string          # Description (optional)
    version: string       # Version constraint (default: "latest")
    install: boolean     # Install this package (default: true)
```

**Example:**
```yaml
npm:
  - name: "@openai/codex"
    desc: OpenAI Codex CLI
    version: "latest"
    install: true
  - name: "@bitwarden/cli"
    desc: Bitwarden CLI for secrets management
    version: "latest"
    install: true
```

**Querying:**
```bash
# List NPM packages
omarchy-config list npm

# Get specific value
omarchy-config get npm
```

---

### scripts

Custom shell scripts to run during installation.

**Schema:**
```yaml
scripts: []  # Currently unused, reserved for future use
```

**Note:** This section is currently empty but reserved for future script execution functionality.

---

### uv_tools

Python tools installed via `uv` (Python package manager).

**Schema:**
```yaml
uv_tools:
  - name: string         # Package name (required)
    desc: string         # Description (optional)
    install: boolean     # Install this tool (default: true)
```

**Example:**
```yaml
uv_tools:
  - name: ruff
    desc: An extremely fast Python linter and formatter
    install: true
  - name: mypy
    desc: Optional static typing for Python
    install: false
```

**Querying:**
```bash
# List uv tools
omarchy-config list uv_tools
```

---

### themes

Theme system configuration.

**Schema:**
```yaml
themes:
  default: string        # Default theme name
  available:            # List of available theme names
    - string
  paths:
    base: string         # Base path for themes (supports ~ expansion)
```

**Example:**
```yaml
themes:
  default: osaka-jade
  available:
    - catppuccin
    - catppuccin-latte
    - nord
    - tokyo-night
    # ... more themes
  paths:
    base: ~/.local/share/omarchy-pop/themes
```

**Querying:**
```bash
# Get default theme
omarchy-config get themes.default

# Get themes base path
omarchy-config get themes.paths.base
```

**See Also:**
- [Theme System Documentation](../themes/README.md)
- [Editor Theming Guide](editor-theming.md)

---

### hp_zbook_ultra

HP ZBook Ultra G1a specific configuration and notes.

**Schema:**
```yaml
hp_zbook_ultra:
  emit_notes: boolean    # Show hardware-specific notes (default: true)
  oem_kernel: string    # OEM kernel version to use
  warn_on_mix: boolean  # Warn if mixing Pop!_OS and Ubuntu kernels
  notes: string         # Multi-line notes about hardware compatibility
```

**Example:**
```yaml
hp_zbook_ultra:
  emit_notes: true
  oem_kernel: linux-oem-24.04c
  warn_on_mix: true
  notes: |
    Pop!_OS uses its own kernel. HP ZBook Ultra G1a hardware status:
    - Fingerprint: Works with fprintd after firmware update
    - Webcam: Requires AMD ISP4 driver
    - WiFi: May have stability issues
```

**Note:** This section is only relevant for HP ZBook Ultra G1a hardware. It provides hardware-specific guidance and kernel recommendations.

---

## Item Schema

Most items in the configuration follow a common schema:

### Required Fields

- `name` or `id`: Identifier for the item
  - APT packages use `name`
  - Flatpak apps use `id`
  - NPM packages use `name` (can include scope)

### Optional Fields

- `install`: Boolean flag (default: `true` if `defaults.install` is `true`)
  - Set to `false` to mark as optional (can install via `omarchy-pop-install`)
- `desc`: Human-readable description
- `version`: Version constraint (NPM packages)
- `alias`: Alternative command name (APT packages)
- `source`: Special source indicator (e.g., `"source"` for Ghostty)
- `note`: Additional notes or warnings
- `url`: Download URL (fonts, installers)
- `method`: Installation method (installers)
- `check`: Command to verify installation (installers)
- `args`: Arguments to pass to installer script (installers)

### Default Behavior

- If `install` is not specified, it defaults to `defaults.install`
- If `defaults.install` is `true`, items default to `install: true`
- If `defaults.install` is `false`, items default to `install: false` unless explicitly set to `true`

---

## Validation Rules

### General Rules

1. **YAML Syntax**: File must be valid YAML
2. **Section Names**: Must match expected section names (case-sensitive)
3. **Group Names**: Must match expected group names within sections
4. **Required Fields**: `name` or `id` is required for all items
5. **Boolean Values**: `install` must be `true` or `false` (not strings)

### Section-Specific Rules

#### apt
- `name` is required
- `alias` is optional (used when package name ≠ command name)
- `source: "source"` triggers source build (requires additional setup)

#### flatpak
- `id` is required (must be valid Flatpak application ID)
- Must use reverse DNS notation (e.g., `com.example.App`)

#### installers
- `method` must be one of: `script`, `custom_*`, `deb`, `npm`, `bun`, `manual`
- `check` command must be provided to verify installation
- Method-specific fields must be provided:
  - `script`: `url` (and optionally `args`)
  - `deb`: `deb_url`
  - `npm`: `npm_package`
  - `bun`: `bun_package`

#### npm
- `name` can include scope (e.g., `@scope/package`)
- `version` defaults to `"latest"` if not specified

---

## Examples

### Minimal Configuration

```yaml
defaults:
  install: true
  theme: nord

apt:
  core:
    - name: git
      install: true
```

### Disabling Default Installation

```yaml
defaults:
  install: false  # Everything defaults to disabled

apt:
  core:
    - name: git
      install: true  # Explicitly enable
    - name: vim
      # install: false (implicit)
```

### Optional Software

```yaml
flatpak:
  utility:
    - id: com.spotify.Client
      desc: Music streaming
      install: false  # Available via omarchy-pop-install
```

### Custom Installer

```yaml
installers:
  runtimes:
    - name: rust
      method: script
      url: https://sh.rustup.rs
      args: -y
      check: rustc
      install: true
```

### Complete Example

See [omarchy-pop.yaml](../omarchy-pop.yaml) in the repository root for a complete example with all sections populated.

---

## Querying Configuration

Use `omarchy-config` to query the configuration:

```bash
# Get a value
omarchy-config get defaults.theme

# List items in a section/group
omarchy-config list apt core

# List only names
omarchy-config list apt core --names-only

# List disabled items
omarchy-config list flatpak utility --disabled

# Output as JSON
omarchy-config list npm --json
```

**See Also:**
- [CLI Reference](cli-reference.md) - Complete `omarchy-config` documentation

---

## Configuration File Location

The configuration file is searched in this order:

1. `--config` flag value
2. `$OMARCHY_POP_CONFIG` environment variable
3. `./omarchy-pop.yaml` (current directory)
4. Repository root `omarchy-pop.yaml`

**Examples:**
```bash
# Use default location
make install

# Use custom config
make install CONFIG_FILE=/path/to/config.yaml

# Using environment variable
export OMARCHY_POP_CONFIG=/path/to/config.yaml
make install
```

---

## Best Practices

1. **Version Control**: Keep `omarchy-pop.yaml` in version control
2. **Comments**: Use YAML comments (`#`) to document choices
3. **Grouping**: Keep related packages in the same group
4. **Descriptions**: Always include `desc` fields for clarity
5. **Optional Items**: Mark truly optional items as `install: false`
6. **Testing**: Use `make dry-run` to preview changes before applying

---

## Troubleshooting

### Configuration Not Found

```bash
# Check if file exists
ls -la omarchy-pop.yaml

# Verify path
omarchy-config --config /path/to/config.yaml get defaults.theme
```

### Invalid YAML

```bash
# Validate YAML syntax
python3 -c "import yaml; yaml.safe_load(open('omarchy-pop.yaml'))"
```

### Query Errors

```bash
# List available sections
omarchy-config list apt  # Will show error with available sections

# List available groups
omarchy-config list apt core  # Will show error with available groups if invalid
```

**See Also:**
- [Troubleshooting Guide](troubleshooting.md)
- [CLI Reference](cli-reference.md)

