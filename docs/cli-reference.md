# CLI Reference

Complete reference for all command-line utilities in the cosmikase project.

## Table of Contents

- [Shell Scripts](#shell-scripts)
  - [cosmikase](#cosmikase)
  - [cosmikase-theme](#cosmikase-theme)
  - [cosmikase-update](#cosmikase-update)
  - [cosmikase-install](#cosmikase-install)
  - [cosmikase-databases](#cosmikase-databases)
  - [cosmikase-cursor-extensions](#cosmikase-cursor-extensions)
  - [cosmikase-power-helper](#cosmikase-power-helper)
- [Python CLI Tools](#python-cli-tools)
  - [cosmikase-cli](#cosmikase-cli)
  - [cosmikase-config](#cosmikase-config)
  - [cosmikase-validate-config](#cosmikase-validate-config)
  - [cosmikase-chezmoi](#cosmikase-chezmoi)
  - [cosmikase-validate-ron](#cosmikase-validate-ron)
  - [cosmikase-themes-dir](#cosmikase-themes-dir)
  - [theme-tui](#theme-tui)

---

## Shell Scripts

### cosmikase

Interactive menu entrypoint for common cosmikase actions.

**Usage:**
```bash
cosmikase
```

**Description:**
Provides a text-based menu interface using `gum` for:
- Theme selection
- Optional software installation
- Docker database setup
- System updates
- Power settings
- Cursor extension management

**Requirements:**
- `gum` must be installed (`sudo apt install gum`)

**Menu Options:**
- **Change Theme**: Launches `theme-tui` if available, otherwise prompts for theme selection
- **Install Optional Software**: Runs `cosmikase-install`
- **Setup Docker Databases**: Runs `cosmikase-databases`
- **Update Everything**: Runs `cosmikase-update`
- **Power Settings**: Runs `cosmikase-power-helper`
- **Cursor Extensions**: Runs `cosmikase-cursor-extensions`
- **Exit**: Closes the menu

**Examples:**
```bash
# Launch interactive menu
cosmikase
```

**Exit Codes:**
- `0`: Success
- `1`: Error (e.g., `gum` not found)

---

### cosmikase-theme

Switch themes by updating chezmoi configuration and applying theme changes to running applications.

**Usage:**
```bash
cosmikase-theme <theme-name> [options]
cosmikase-theme --rollback
```

**Description:**
Switches the active theme by:
1. Updating `~/.config/chezmoi/chezmoi.toml` with the new theme
2. Running `chezmoi apply` to regenerate dotfiles
3. Calling helper scripts to update running applications (Cursor, COSMIC, terminals)

**Arguments:**
- `<theme-name>`: Name of the theme to switch to (e.g., `nord`, `tokyo-night`, `catppuccin`)

**Options:**
- `--rollback`: Roll back to the previous theme (uses theme history)
- `--no-cursor`: Skip Cursor/VS Code theme update
- `--no-cosmic`: Skip COSMIC desktop theme update
- `--no-terminals`: Skip terminal reload signals
- `--no-chezmoi`: Skip chezmoi apply (only run helper scripts for live updates)
- `--quiet`, `-q`: Suppress output from helper scripts
- `-h`, `--help`: Show help message

**Examples:**
```bash
# Switch to nord theme
cosmikase-theme nord

# Switch theme but skip Cursor update
cosmikase-theme tokyo-night --no-cursor

# Only update running apps (don't regenerate dotfiles)
cosmikase-theme catppuccin --no-chezmoi

# Roll back to previous theme
cosmikase-theme --rollback

# Quiet mode (minimal output)
cosmikase-theme nord --quiet
```

**Theme History:**
The script maintains a history file at `~/.config/cosmikase/theme-history` to enable rollback functionality.

**Exit Codes:**
- `0`: Success
- `1`: Error (invalid theme, chezmoi not found, etc.)

**See Also:**
- [Theme System Documentation](../themes/README.md)
- [Editor Theming Guide](editor-theming.md)

---

### cosmikase-update

Updates all installed software and system components.

**Usage:**
```bash
cosmikase-update
```

**Description:**
Comprehensive system update script that updates:
- APT packages (`apt update && apt full-upgrade`)
- Flatpak applications
- Snap packages (if snapd is active)
- Rust toolchain (via `rustup`)
- uv Python package manager
- Bun JavaScript runtime
- Julia (via `juliaup`)
- Global NPM packages
- Ghostty terminal (if built from source)
- Firmware (via `fwupdmgr`)

**Requirements:**
- `sudo` access for system package updates
- Internet connection

**What It Updates:**
1. **System Packages**: APT, Flatpak, Snap
2. **Runtimes**: Rust, Bun, Julia, Node.js (via npm)
3. **Package Managers**: uv (installs if missing)
4. **Ghostty**: Rebuilds from source if `~/ghostty-source` exists
5. **Firmware**: Checks for firmware updates (does not install automatically)

**Examples:**
```bash
# Run full system update
cosmikase-update
```

**Notes:**
- Firmware updates require manual confirmation: `sudo fwupdmgr update`
- Ghostty rebuild requires Zig 0.13+ to be installed
- uv will be installed automatically if missing

**Exit Codes:**
- `0`: Success
- Non-zero: Error during update process

---

### cosmikase-install

Interactive installer for optional software marked as `install: false` in `cosmikase.yaml`.

**Usage:**
```bash
cosmikase-install [--config PATH]
```

**Description:**
Scans the configuration file for items marked `install: false` and allows interactive selection and installation via `gum`.

**Options:**
- `--config PATH`: Path to `cosmikase.yaml` (default: `./cosmikase.yaml` or `$COSMIKASE_CONFIG`)
- `-h`, `--help`: Show help message

**Environment Variables:**
- `COSMIKASE_CONFIG`: Path to configuration file

**Examples:**
```bash
# Install optional software (run from repo root)
cosmikase-install

# Use custom config file
cosmikase-install --config /path/to/cosmikase.yaml

# Using environment variable
export COSMIKASE_CONFIG=/path/to/cosmikase.yaml
cosmikase-install
```

**What It Installs:**
- APT packages from `apt.core`, `apt.gui`, `apt.terminal` sections
- Flatpak applications from `flatpak.utility` section
- Only items with `install: false` are shown for selection

**Removing Installed Software:**
```bash
# APT packages
sudo apt remove <package-name>

# Flatpak applications
flatpak uninstall <app-id>
```

**Requirements:**
- `gum` must be installed
- `cosmikase-config` must be available (or `uv` for fallback)

**Exit Codes:**
- `0`: Success
- `1`: Error (config not found, dependencies missing)

**See Also:**
- [Configuration Reference](configuration-reference.md)
- [cosmikase-menu.md](cosmikase-menu.md)

---

### cosmikase-databases

Setup development databases via Docker containers.

**Usage:**
```bash
cosmikase-databases
```

**Description:**
Interactive script to create and start Docker containers for common development databases:
- PostgreSQL (port 5432)
- MySQL (port 3306)
- Redis (port 6379)
- MongoDB (port 27017)

**Requirements:**
- `docker` must be installed and running
- `gum` must be installed

**What It Creates:**
- Docker containers named `cosmikase-postgres`, `cosmikase-mysql`, `cosmikase-redis`, `cosmikase-mongodb`
- Docker volumes for data persistence:
  - `cosmikase-postgres-data`
  - `cosmikase-mysql-data`
  - `cosmikase-redis-data`
  - `cosmikase-mongodb-data`

**Connection Strings:**
- **PostgreSQL**: `postgres://postgres:<password>@localhost:5432/postgres`
- **MySQL**: `mysql -h 127.0.0.1 -P 3306 -u root -p`
- **Redis**: `redis-cli -h 127.0.0.1 -p 6379`
- **MongoDB**: `mongosh mongodb://127.0.0.1:27017`

**Examples:**
```bash
# Launch interactive database setup
cosmikase-databases

# Verify containers are running
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

**Cleanup:**
```bash
# Stop and remove containers
docker rm -f cosmikase-postgres cosmikase-mysql cosmikase-redis cosmikase-mongodb

# Remove volumes (deletes all data)
docker volume rm cosmikase-postgres-data cosmikase-mysql-data cosmikase-redis-data cosmikase-mongodb-data
```

**Exit Codes:**
- `0`: Success
- `1`: Error (docker/gum not found, password empty)

**See Also:**
- [cosmikase-menu.md](cosmikase-menu.md)

---

### cosmikase-cursor-extensions

Manage Cursor/VS Code extensions from a text file.

**Usage:**
```bash
cosmikase-cursor-extensions <command> [-f FILE]
```

**Description:**
Manages Cursor/VS Code extensions via a simple text file, making it easy to:
- Export current extensions
- Install extensions on a new machine
- Compare installed vs. listed extensions

**Commands:**
- `install`: Install all extensions from the list
- `export`: Export currently installed extensions to the list
- `list`: Show extensions that would be installed
- `diff`: Show extensions in list but not installed (and vice versa)

**Options:**
- `-f FILE`: Use a different extensions file (default: `~/.config/Cursor/extensions.txt`)
- `-h`, `--help`: Show help message

**Environment Variables:**
- `EXTENSIONS_FILE`: Path to extensions file (default: `~/.config/Cursor/extensions.txt`)
- `CURSOR_CMD`: Command to use (default: `cursor`, falls back to `code`)

**Examples:**
```bash
# Export current extensions
cosmikase-cursor-extensions export

# Install all extensions from list
cosmikase-cursor-extensions install

# See what's different
cosmikase-cursor-extensions diff

# List extensions in file
cosmikase-cursor-extensions list

# Use custom file
cosmikase-cursor-extensions install -f ~/my-extensions.txt
```

**Extensions File Format:**
The extensions file (`~/.config/Cursor/extensions.txt`) is a simple text file with one extension ID per line:
```
# Cursor/VS Code Extensions
# Exported on 2025-01-27 12:00:00
# Install with: cosmikase-cursor-extensions install

catppuccin.catppuccin-vsc
ms-python.python
ms-python.vscode-pylance
```

**Exit Codes:**
- `0`: Success
- `1`: Error (file not found, command not found)

**See Also:**
- [Editor Theming Guide](editor-theming.md)

---

### cosmikase-power-helper

Switch power profiles based on AC/Battery state.

**Usage:**
```bash
cosmikase-power-helper [--ac|--battery|--apply]
```

**Description:**
Helper script for managing System76 power profiles. Typically called by udev rules when AC power state changes, but can also be run manually.

**Options:**
- `--ac`: Set power profile to performance (or balanced if performance unavailable)
- `--battery`: Set power profile to battery
- `--apply` (default): Auto-detect AC state and apply appropriate profile

**Examples:**
```bash
# Auto-detect and apply profile
cosmikase-power-helper

# Force performance mode
cosmikase-power-helper --ac

# Force battery mode
cosmikase-power-helper --battery
```

**Udev Integration:**
The script is designed to be called by udev rules (see `cosmikase/udev/99-cosmikase-power.rules`). When AC power is plugged/unplugged, udev triggers this script to switch profiles automatically.

**Requirements:**
- `system76-power` must be installed
- Script must be executable and on PATH

**Exit Codes:**
- `0`: Success
- `1`: Failed to set profile

**See Also:**
- System76 Power Management documentation

---

## Python CLI Tools

### cosmikase-cli

Unified Python CLI for cosmikase (separate from the interactive `cosmikase` menu).

**Usage:**
```bash
cosmikase-cli <command> [options]
```

**Commands:**
- `theme` (switch themes; supports `--list`, `--no-apply`, `--no-helpers`)
- `config` (query configuration values)
- `validate` (validate configuration file)
- `themes-dir` (print theme directories)

**Examples:**
```bash
cosmikase-cli theme nord
cosmikase-cli theme --list
cosmikase-cli config defaults.theme
cosmikase-cli validate cosmikase.yaml
cosmikase-cli themes-dir --all
```

**Notes:**
- Use `cosmikase` for the interactive menu.

### cosmikase-config

Query the `cosmikase.yaml` configuration file from shell scripts.

**Usage:**
```bash
cosmikase-config [--config PATH] <command> [options]
```

**Description:**
Python CLI tool for querying the YAML configuration file. Used by shell scripts to extract configuration values.

**Global Options:**
- `--config PATH`, `-c PATH`: Path to config file (default: `cosmikase.yaml`)

**Commands:**

#### `get`
Get a value by dot-separated path.

```bash
cosmikase-config get <path> [--default VALUE]
```

**Options:**
- `--default VALUE`, `-d VALUE`: Default value if path not found

**Examples:**
```bash
# Get default theme
cosmikase-config get defaults.theme

# Get theme with default
cosmikase-config get defaults.theme --default nord

# Get boolean value
cosmikase-config get defaults.install
# Output: true or false
```

#### `list`
List items from a section/group.

```bash
cosmikase-config list <section> [group] [options]
```

**Arguments:**
- `section`: Section name (e.g., `apt`, `flatpak`, `runtimes`)
- `group`: Optional group name within section (e.g., `core`, `utility`)

**Options:**
- `--names-only`, `-n`: Output only package names (one per line)
- `--json`, `-j`: Output as JSON
- `--all`, `-a`: Include disabled items (`install: false`)
- `--disabled`, `-d`: Show ONLY disabled items

**Examples:**
```bash
# List enabled apt core packages
cosmikase-config list apt core

# List only names
cosmikase-config list apt core --names-only

# List all (including disabled)
cosmikase-config list apt core --all

# List only disabled items
cosmikase-config list apt core --disabled

# Output as JSON
cosmikase-config list flatpak utility --json

# List top-level section (no group)
cosmikase-config list npm
```

**Exit Codes:**
- `0`: Success
- `1`: Error (config not found, invalid section/group)

**See Also:**
- [Configuration Reference](configuration-reference.md)

---

### cosmikase-validate-config

Validate `cosmikase.yaml` against the Pydantic schema.

**Usage:**
```bash
cosmikase-validate-config [path] [options]
```

**Options:**
- `-q`, `--quiet`: Only print errors.

**Examples:**
```bash
cosmikase-validate-config
cosmikase-validate-config /path/to/cosmikase.yaml --quiet
```

**Exit Codes:**
- `0`: Valid configuration
- `1`: Validation errors

### cosmikase-chezmoi

Update chezmoi configuration with theme information.

**Usage:**
```bash
cosmikase-chezmoi <theme> <themes-dir>
```

**Description:**
Internal utility used by `cosmikase-theme` to safely update `~/.config/chezmoi/chezmoi.toml` with theme data. Performs atomic writes to prevent corruption.

**Arguments:**
- `theme`: Theme name to set
- `themes-dir`: Path to themes directory

**What It Updates:**
Updates the `[data]` section in `chezmoi.toml`:
```toml
[data]
theme = "nord"
themes_dir = "/path/to/themes"
```

**Examples:**
```bash
# Update chezmoi config (typically called by cosmikase-theme)
cosmikase-chezmoi nord ~/.local/share/cosmikase/themes
```

**Exit Codes:**
- `0`: Success
- `1`: Error (invalid TOML, permission denied, write failed)

**Note:** This command is typically called automatically by `cosmikase-theme`. Direct usage is rarely needed.

---

### cosmikase-validate-ron

Validate RON (Rusty Object Notation) file syntax.

**Usage:**
```bash
cosmikase-validate-ron <path>
```

**Description:**
Basic RON syntax validator that checks for balanced parentheses, brackets, and braces. Used to validate COSMIC theme files.

**Arguments:**
- `path`: Path to RON file to validate

**Examples:**
```bash
# Validate a COSMIC theme file
cosmikase-validate-ron ~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark

# Validate theme file
cosmikase-validate-ron themes/nord/cosmic.ron
```

**Limitations:**
This is a basic validator that checks bracket/parenthesis balance. It does not perform full RON parsing or semantic validation.

**Exit Codes:**
- `0`: File is valid (basic check passed)
- `1`: File is invalid or not found

**See Also:**
- [COSMIC Theming Guide](cosmic-theming.md)

---

### cosmikase-themes-dir

Discover and print the themes directory path.

**Usage:**
```bash
cosmikase-themes-dir [--all|--list]
```

**Description:**
Discovers the themes directory by checking multiple locations and prints the primary path. Used by shell scripts to locate theme files.

**Options:**
- `--all`, `-a`: Print all discovered theme directories (one per line)
- `--list`, `-l`: List available themes in the primary directory

**Search Order:**
1. `$THEMES_DIR` environment variable
2. `themes/` directory in repo root
3. `./themes` in current directory
4. `~/.local/share/cosmikase/themes`

**Examples:**
```bash
# Print primary themes directory
cosmikase-themes-dir
# Output: /home/user/.local/share/cosmikase/themes

# List all theme directories
cosmikase-themes-dir --all

# List available themes
cosmikase-themes-dir --list
```

**Exit Codes:**
- `0`: Success
- `1`: No theme directories found

**See Also:**
- [Theme System Documentation](../themes/README.md)

---

### theme-tui

Interactive terminal UI for browsing and applying themes.

**Usage:**
```bash
theme-tui
```

**Description:**
Text-based user interface built with Textual for browsing available themes, previewing colors, and applying themes interactively.

**Features:**
- Browse all available themes
- Preview theme colors and metadata
- Apply theme with Enter key
- Navigate with arrow keys
- Exit with `q` or `Ctrl+C`

**Keyboard Shortcuts:**
- `↑` / `↓`: Navigate theme list
- `Enter`: Apply selected theme
- `q`: Quit
- `Ctrl+C`: Quit

**Requirements:**
- Python 3.10+
- Textual library (installed via `uv sync`)
- `cosmikase-theme` script must be available

**Examples:**
```bash
# Launch theme browser
theme-tui
```

**What It Shows:**
- Theme name
- Variant (dark/light)
- Color swatches (background, foreground, accent, error, warning)
- Cursor theme name
- Wallpaper path

**Exit Codes:**
- `0`: Success (theme applied or user quit)
- Non-zero: Error

**See Also:**
- [Theme System Documentation](../themes/README.md)
- [Editor Theming Guide](editor-theming.md)

---

## Common Patterns

### Finding Command Locations

Most commands are installed to `~/.local/bin` after running `make install`. Verify with:

```bash
which cosmikase-theme
which cosmikase-config
```

### Using uv Run

If commands aren't on PATH, use `uv run`:

```bash
uv run cosmikase-config list apt core
uv run theme-tui
```

### Environment Variables

Several commands respect environment variables:

```bash
# Themes directory
export THEMES_DIR=/custom/path/to/themes

# Config file
export COSMIKASE_CONFIG=/path/to/config.yaml

# Extensions file
export EXTENSIONS_FILE=~/.config/Cursor/extensions.txt

# Cursor command
export CURSOR_CMD=cursor
```

---

## Getting Help

Most commands support `--help` or `-h`:

```bash
cosmikase-theme --help
cosmikase-cli --help
cosmikase-config --help
cosmikase-cursor-extensions --help
```

For more information, see:
- [README.md](../README.md) - Project overview
- [Configuration Reference](configuration-reference.md) - Config file schema
- [Troubleshooting Guide](troubleshooting.md) - Common issues

