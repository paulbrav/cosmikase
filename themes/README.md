# Cosmikase Themes for Pop!_OS

This directory contains a comprehensive collection of themes optimized for both Pop!_OS and COSMIC desktop environments.

## Available Themes

### Your Custom Themes

- **pop-default** - Pop!_OS-inspired palette with signature orange and teal accents
- **catppuccin** - Dark Catppuccin variant with pastel colors  
- **osaka-jade** - Cyan and jade green aesthetic inspired by Osaka

### Official Themes

- **tokyo-night** - Flagship theme with deep blues and vibrant colors
- **nord** - Cool northern palette with Arctic-inspired colors
- **gruvbox** - Retro warm colors with earthy tones
- **kanagawa** - Japanese-inspired with muted natural colors
- **everforest** - Forest green aesthetic with comfortable colors
- **rose-pine** - Rosé Pine color scheme with soft pastels
- **catppuccin-latte** - Light Catppuccin variant (distinct from dark catppuccin)
- **matte-black** - Minimalist dark theme with high contrast
- **ristretto** - Coffee-inspired warm theme
- **ethereal** - Dreamy ethereal color palette
- **flexoki-light** - Light theme with warm, paper-like aesthetics
- **hackerman** - Matrix-inspired green terminal theme

## Theme File Structure

Each theme directory contains configuration files for various applications:

### Terminal Emulators
- `alacritty.toml` - Alacritty terminal color scheme
- `ghostty.conf` - Ghostty terminal color scheme
- `kitty.conf` - Kitty terminal color scheme

### Desktop Environment (Hyprland/Wayland)
- `hyprland.conf` - Hyprland compositor theming
- `hyprlock.conf` - Hyprlock screen lock styling
- `waybar.css` - Waybar status bar styling
- `mako.ini` - Mako notification daemon styling
- `swayosd.css` - SwayOSD on-screen display styling
- `walker.css` - Walker app launcher styling

### Development Tools
- `nvim.lua` / `neovim.lua` - Neovim color scheme
- `btop.theme` - Resource monitor theming
- `starship.toml` - Shell prompt configuration

### Editor/IDE Integration
- `cursor.json` - Cursor/VS Code theme config (see schema below)
- `antigravity.conf` - Antigravity launcher colors

### System Integration
- `theme.yaml` - Unified theme manifest (replaces legacy indicators)
- `cosmic.ron` - COSMIC desktop theme
- `cosmic-term.ron` - COSMIC Terminal theme
- `icons.theme` - Preferred icon theme name
- `chromium.theme` - Browser theme color (RGB format)
- `light.mode` - Legacy light theme indicator (obsolete in v0.3)

### Wallpapers
- `backgrounds/` - Directory containing wallpaper images
- `preview.png` - Theme preview image (official themes)

## Usage

### On Pop!_OS (GNOME/COSMIC)

The following theme files work directly on Pop!_OS without Hyprland:

- Terminal configs (alacritty, ghostty, kitty)
- Development tools (nvim, btop, starship)
- System integration (cursor, icons, chromium)
- Wallpapers

### With Hyprland Installed

If you install Hyprland on Pop!_OS, you can use the full theme experience:

- All terminal and development configs
- Hyprland compositor theming
- Waybar status bar
- Mako notifications
- Walker launcher
- Full desktop environment theming

### Applying Themes

To switch themes, update your configuration files to import the desired theme:

**For Alacritty** (`~/.config/alacritty/alacritty.toml`):
```toml
import = ["/path/to/cosmikase/themes/tokyo-night/alacritty.toml"]
```

**For Ghostty** (`~/.config/ghostty/config`):
```
import = /path/to/cosmikase/themes/tokyo-night/ghostty.conf
```

**For Neovim**, source the theme file in your init.lua:
```lua
dofile("/path/to/cosmikase/themes/tokyo-night/nvim.lua")
```

## theme.yaml Schema (v0.3+)

Each theme directory contains a `theme.yaml` file that defines its metadata and key properties:

```yaml
name: Catppuccin Mocha
variant: dark              # dark or light
colors:
  background: "#1e1e2e"    # hex color code
  foreground: "#cdd6f4"
  accent: "#89b4fa"
  error: "#f38ba8"
  warning: "#f9e2af"
cursor:
  theme: Catppuccin Mocha  # VS Code colorTheme name
  extension: catppuccin.catppuccin-vsc
wallpaper: backgrounds/cat_mountains.png  # Path relative to theme dir
```

The theme system uses this manifest to:
- Generate color previews in the TUI
- Set the correct `workbench.colorTheme` in Cursor/VS Code
- Determine dark/light mode for COSMIC
- Select the default wallpaper

## Legacy Migration (v0.2 to v0.3)

If you have custom themes from v0.2, you can migrate them using the included script:

```bash
uv run python scripts/migrate-themes.py
```

This will automatically generate a `theme.yaml` for each theme directory based on existing `cursor.json`, `light.mode`, and background files.

## cursor.json Schema (Legacy)

```json
{
  "colorTheme": "Theme Name",      // VS Code theme name (required)
  "extension": "publisher.ext-id", // Extension ID (null if built-in)
  "light": true,                   // Light theme flag (optional, default: false)
  "colors": {
    "background": "#1e1e2e",       // Primary background color
    "foreground": "#cdd6f4",       // Primary text color
    "accent": "#89b4fa",           // Accent/highlight color
    "error": "#f38ba8",            // Error color
    "warning": "#f9e2af"           // Warning color
  }
}
```

The chezmoi templates read from this file to:
- Set the correct `workbench.colorTheme` in Cursor settings
- Apply colors to Antigravity launcher
- Generate consistent color overrides across tools

## antigravity.conf Format

Simple key-value format for the Antigravity launcher:

```ini
# Theme Name palette
background=#1e1e2e
foreground=#cdd6f4
accent=#89b4fa
warning=#f9e2af
error=#f38ba8
```

## Theme Consistency

All themes maintain consistent color palettes across applications:
- Primary colors for active elements and highlights
- Secondary colors for borders and accents
- Background colors for surfaces and containers
- Text colors optimized for readability

## Customization

Feel free to:
- Modify existing themes to match your preferences
- Create new theme directories following the same structure
- Mix and match configurations from different themes
- Add wallpapers to the `backgrounds/` directories

## Credits

- Official themes: [basecamp/omarchy](https://github.com/basecamp/omarchy) (source repository)
- Custom Pop!_OS themes: Created for cosmikase project
- Catppuccin: [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin)
- Nord: [nordtheme/nord](https://github.com/nordtheme/nord)
- Gruvbox: [morhetz/gruvbox](https://github.com/morhetz/gruvbox)
- Tokyo Night: [tokyo-night](https://github.com/enkia/tokyo-night-vscode-theme)
- Rose Pine: [rose-pine](https://github.com/rose-pine/rose-pine-theme)
