# Editor Theming Guide

A comprehensive guide to theming Cursor and Antigravity within the omarchy ecosystem, covering both automated theme switching and manual configuration.

## Table of Contents

- [Omarchy Theme System Overview](#omarchy-theme-system-overview)
- [Cursor Theming](#cursor-theming)
- [Antigravity Theming](#antigravity-theming)
- [Adding a New Theme](#adding-a-new-theme)
- [Manual Cursor Configuration](#manual-cursor-configuration)
- [Troubleshooting](#troubleshooting)
- [References](#references)

---

## Omarchy Theme System Overview

The omarchy system provides unified theming across multiple applications with a single command. When you run `omarchy-pop-theme`, it updates configurations for COSMIC desktop, terminals (Ghostty, Kitty, Alacritty), Cursor, Antigravity, and other tools simultaneously.

### How Theme Switching Works

```bash
omarchy-pop-theme <theme-name>
```

This command:
1. Updates `~/.config/chezmoi/chezmoi.toml` with the new theme name
2. Runs `chezmoi apply` to regenerate all templated config files
3. Applications pick up changes via their own mechanisms (inotify, reload, etc.)

### Theme Directory Structure

Each theme lives in `themes/<name>/` with application-specific files:

```
themes/catppuccin/
├── antigravity.conf    # Antigravity color palette
├── cursor.json         # Cursor/VS Code color metadata
├── ghostty.conf        # Ghostty terminal colors
├── kitty.conf          # Kitty terminal colors
├── cosmic.ron          # COSMIC desktop theme
├── btop.theme          # btop system monitor
└── ...
```

### Chezmoi Template Flow

Chezmoi templates in `chezmoi/dot_config/` reference the active theme:

| Template | Destination | Purpose |
|----------|-------------|---------|
| `Cursor/User/settings.json.tmpl` | `~/.config/Cursor/User/settings.json` | Sets `workbench.colorTheme` |
| `Cursor/theme.json.tmpl` | `~/.config/Cursor/theme.json` | Theme metadata |
| `antigravity/config.toml.tmpl` | `~/.config/antigravity/config.toml` | Points to theme file |

Templates use `.theme` and `.themes_dir` variables from chezmoi config.

---

## Cursor Theming

Cursor is a VS Code fork, so it uses VS Code's theming system with `workbench.colorTheme` in settings.

### How Omarchy Themes Cursor

The template `chezmoi/dot_config/Cursor/User/settings.json.tmpl` maps omarchy theme names to VS Code theme extensions:

```jsonc
{{- if eq .theme "catppuccin" }}
    "workbench.colorTheme": "Catppuccin Mocha",
{{- else if eq .theme "nord" }}
    "workbench.colorTheme": "Nord",
{{- else if eq .theme "tokyo-night" }}
    "workbench.colorTheme": "Tokyo Night",
// ... more mappings
{{- end }}
```

### Theme-to-Extension Mapping

| Omarchy Theme | VS Code Theme Extension |
|---------------|------------------------|
| `catppuccin` | Catppuccin Mocha |
| `catppuccin-latte` | Catppuccin Latte |
| `nord` | Nord |
| `tokyo-night` | Tokyo Night |
| `gruvbox` | Gruvbox Dark Hard |
| `everforest` | Everforest Dark |
| `kanagawa` | Kanagawa |
| `rose-pine` | Rosé Pine Dawn |
| `flexoki-light` | Flexoki Light |
| `hackerman` | SynthWave '84 |
| `matte-black` | Abyss |
| `ethereal` | One Dark Pro |
| `ristretto` | Monokai Pro |
| `pop-default` | Pop Dark |

### Installing Required Extensions

Theme extensions must be installed for themes to display correctly:

```bash
# Install all extensions from the managed list
omarchy-cursor-extensions install

# Or install a specific theme extension
cursor --install-extension catppuccin.catppuccin-vsc
```

The extension list is managed in `~/.config/Cursor/extensions.txt`.

### After Theme Switch

When you run `omarchy-pop-theme`, Cursor needs a window reload:

1. Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
2. Type "reload"
3. Select **"Developer: Reload Window"**

---

## Antigravity Theming

Antigravity is an AI-powered terminal tool. It reads its theme configuration from a TOML file that points to a color palette.

### Configuration Location

```
~/.config/antigravity/config.toml
```

### Template Structure

The chezmoi template (`chezmoi/dot_config/antigravity/config.toml.tmpl`):

```toml
# Antigravity configuration
# Theme is dynamically applied via chezmoi template

[theme]
file = "{{ .themes_dir }}/{{ .theme }}/antigravity.conf"
```

### Theme File Format

Each theme provides `antigravity.conf` with key-value color definitions:

```conf
# Catppuccin palette
background=#1e1e2e
foreground=#cdd6f4
accent=#89b4fa
warning=#f9e2af
error=#f38ba8
```

Available color keys:
- `background` — main background color
- `foreground` — primary text color
- `accent` — highlight/accent color
- `warning` — warning indicator color
- `error` — error indicator color

### Verifying Theme Application

Check that the config points to your theme:

```bash
cat ~/.config/antigravity/config.toml
```

Should show the path to your active theme's `antigravity.conf`.

---

## Adding a New Theme

To add a new theme to the omarchy system:

### 1. Create Theme Directory

```bash
mkdir -p themes/my-new-theme
```

### 2. Create Required Files

**`themes/my-new-theme/cursor.json`**:

```json
{
  "name": "My New Theme",
  "background": "#1a1b26",
  "foreground": "#c0caf5",
  "accent": "#7aa2f7",
  "error": "#f7768e",
  "warning": "#e0af68"
}
```

**`themes/my-new-theme/antigravity.conf`**:

```conf
# My New Theme palette
background=#1a1b26
foreground=#c0caf5
accent=#7aa2f7
warning=#e0af68
error=#f7768e
```

### 3. Map Cursor Theme Extension

Edit `chezmoi/dot_config/Cursor/User/settings.json.tmpl` to add your theme mapping:

```jsonc
{{- else if eq .theme "my-new-theme" }}
    "workbench.colorTheme": "My Theme Extension Name",
```

The extension name must match exactly what appears in Cursor's Color Theme picker.

### 4. Install the VS Code Theme Extension

Add to `chezmoi/dot_config/Cursor/extensions.txt`:

```
publisher.my-theme-extension-id
```

Then install:

```bash
omarchy-cursor-extensions install
```

### 5. Test the Theme

```bash
omarchy-pop-theme my-new-theme
```

---

## Manual Cursor Configuration

For one-off theme changes or customization without modifying omarchy templates.

### Quick Theme Change (Command Palette)

1. Open Cursor
2. Open Command Palette: `Ctrl+Shift+P` (Linux/Windows) or `Cmd+Shift+P` (macOS)
3. Type `theme`
4. Select **"Preferences: Color Theme"**
5. Use ↑/↓ to preview, press **Enter** to confirm

### Installing Themes from Extensions

1. Open Extensions: `Ctrl+Shift+X` or `View → Extensions`
2. Search for a theme (e.g., "Catppuccin")
3. Click **Install**
4. Open Command Palette → **"Preferences: Color Theme"** → select installed theme

### CLI Installation

```bash
cursor --install-extension diogomoretti.hexxa-theme
```

### Setting Theme in settings.json

For a persistent default (useful for syncing via dotfiles):

1. Open Command Palette
2. Run **"Preferences: Open User Settings (JSON)"**
3. Add or edit:

```jsonc
{
  "workbench.colorTheme": "Your Theme Name"
}
```

> **Note**: If you're using omarchy themes, edits to `settings.json` will be overwritten on the next `chezmoi apply`. Edit the template instead for permanent changes.

### Custom Color Overrides

Layer custom colors on top of any theme using `workbench.colorCustomizations`:

```jsonc
{
  "workbench.colorTheme": "Catppuccin Mocha",

  "workbench.colorCustomizations": {
    "editor.background": "#050608",
    "editor.foreground": "#E0E6F0",
    "terminal.background": "#050608",
    "terminal.foreground": "#C0CAD8",
    "activityBar.background": "#050608",
    "sideBar.background": "#050608"
  }
}
```

### Token (Syntax) Color Overrides

Override syntax highlighting colors:

```jsonc
{
  "editor.tokenColorCustomizations": {
    "comments": "#6B7280",
    "keywords": "#F97373"
  }
}
```

---

## Troubleshooting

### Cursor Theme Not Applying

**Symptom**: Theme name is set but colors don't change.

**Solutions**:
1. Reload the window: `Ctrl+Shift+P` → "Developer: Reload Window"
2. Verify the theme extension is installed: `View → Extensions` → search for theme name
3. Check the exact theme name matches in settings.json (case-sensitive)

### Cursor Shows Wrong Theme After omarchy-pop-theme

**Symptom**: You ran `omarchy-pop-theme` but Cursor still shows old theme.

**Solutions**:
1. Reload window in Cursor
2. Check chezmoi applied correctly: `chezmoi diff`
3. Verify settings.json was updated: `cat ~/.config/Cursor/User/settings.json`

### Theme Extension Not Found

**Symptom**: Settings reference a theme that doesn't exist.

**Solution**: Install the extension first:

```bash
# Check what's in the managed extension list
cat ~/.config/Cursor/extensions.txt

# Install all managed extensions
omarchy-cursor-extensions install
```

### Antigravity Not Picking Up Theme

**Symptom**: Antigravity shows default/wrong colors.

**Solutions**:
1. Verify config exists: `cat ~/.config/antigravity/config.toml`
2. Check the theme file path is correct and file exists
3. Restart Antigravity to reload configuration

### Checking Current Theme Configuration

```bash
# View chezmoi data (active theme)
chezmoi data | grep theme

# View generated Cursor settings
cat ~/.config/Cursor/User/settings.json | grep colorTheme

# View Antigravity config
cat ~/.config/antigravity/config.toml
```

---

## References

1. [Cursor Themes Documentation](https://cursor.com/docs/configuration/themes) — Official Cursor theming docs
2. [Cursor Community Forum: Theme Colors](https://forum.cursor.com/t/can-i-change-the-theme-color-in-cursor/105) — Community discussion on theming
3. [Cursor Terminal Colors](https://forum.cursor.com/t/are-terminal-colors-themes-supported/71872) — Terminal color customization
4. [JetBrains Migration Guide](https://cursor.com/docs/configuration/migrations/jetbrains) — JetBrains-style themes in Cursor
5. [VS Code Color Theme Reference](https://code.visualstudio.com/docs/getstarted/themes) — Underlying theme system
6. [chezmoi Documentation](https://www.chezmoi.io/) — Dotfile manager used for templating





