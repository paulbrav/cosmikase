# Omarchy Themes for Pop!_OS

This directory contains a comprehensive collection of themes optimized for both Pop!_OS and Omarchy desktop environments.

## Available Themes

### Your Custom Themes

- **pop-default** - Pop!_OS-inspired palette with signature orange and teal accents
- **catppuccin** - Dark Catppuccin variant with pastel colors  
- **osaka-jade** - Cyan and jade green aesthetic inspired by Osaka

### Official Omarchy Themes

- **tokyo-night** - Omarchy's flagship theme with deep blues and vibrant colors
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
- `vscode.json` - VS Code color theme (where available)

### System Integration
- `cursor.json` - Cursor theme definitions
- `icons.theme` - Preferred icon theme name
- `chromium.theme` - Browser theme color (RGB format)
- `light.mode` - Light theme indicator (if present)
- `antigravity.conf` - Custom launcher configuration

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
import = ["/path/to/omarchy-for-popos/themes/tokyo-night/alacritty.toml"]
```

**For Ghostty** (`~/.config/ghostty/config`):
```
import = /path/to/omarchy-for-popos/themes/tokyo-night/ghostty.conf
```

**For Neovim**, source the theme file in your init.lua:
```lua
dofile("/path/to/omarchy-for-popos/themes/tokyo-night/nvim.lua")
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

- Official Omarchy themes: [basecamp/omarchy](https://github.com/basecamp/omarchy)
- Custom Pop!_OS themes: Created for omarchy-for-popos project
- Catppuccin: [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin)
- Nord: [nordtheme/nord](https://github.com/nordtheme/nord)
- Gruvbox: [morhetz/gruvbox](https://github.com/morhetz/gruvbox)
- Tokyo Night: [tokyo-night](https://github.com/enkia/tokyo-night-vscode-theme)
- Rose Pine: [rose-pine](https://github.com/rose-pine/rose-pine-theme)
