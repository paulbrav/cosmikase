# COSMIC Desktop Theming Guide

A comprehensive guide to theming the Rust-based COSMIC desktop environment on Pop!_OS 24.x, covering both GUI controls and programmatic configuration via RON files.

## Table of Contents

- [Mental Model: Where COSMIC Stores Settings](#mental-model-where-cosmic-stores-settings)
- [Changing Theme / Desktop Appearance (GUI)](#changing-theme--desktop-appearance-gui)
- [Changing Wallpaper](#changing-wallpaper)
- [Dock & Panel: GUI Controls](#dock--panel-gui-controls)
- [Dock & Panel: Programmatic Tweaks (RON Files)](#dock--panel-programmatic-tweaks-ron-files)
- [Programmatically Querying / Reacting to Theme](#programmatically-querying--reacting-to-theme)
- [Practical Workflow Suggestion](#practical-workflow-suggestion)
- [References](#references)

---

## Mental Model: Where COSMIC Stores Settings

COSMIC doesn't use `gsettings` for appearance. It has its own configuration system:

| Location | Purpose |
|----------|---------|
| `/usr/share/cosmic` | System defaults (RON files) |
| `~/.config/cosmic/` | **User overrides** (main place to edit/sync) |
| `~/.local/state/cosmic/` | Ephemeral state, mirrors config structure |

### Per-Component Files

Settings are organized by component with names like:
- `~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/...` — Panel configuration
- `~/.config/cosmic/com.system76.CosmicTheme.*` — Theming
- `~/.config/cosmic/com.system76.CosmicBackground` — Wallpapers

Files are mostly tiny RON snippets (often one value per file). `cosmic-settings-daemon` watches them with `inotify` and live-applies changes.

**Pattern**: Set it once in the GUI, inspect what got written into `~/.config/cosmic`, then script/edit that.

---

## Changing Theme / Desktop Appearance (GUI)

### Basic Theme & Colors

1. Open **COSMIC Settings**
2. Go to **Desktop → Appearance**
3. Available options:
   - Switch **Light / Dark** mode
   - Set an **accent color** (picker or presets)
   - Adjust **application window background**, **container background**, **text tint**, **neutral tint**
   - Choose **corner radius style** (round / slightly round / square)
   - Tweak **interface density** and tiling gaps / active-window hint width

The "Experimental" or advanced section allows applying the theme to GTK apps, changing icon theme, and **export/import** appearance settings as a `.ron` theme file.

### Icon, Font and Distro Branding

**Icon themes**:
- Install theme into `/usr/share/icons` (system-wide) or `~/.local/share/icons` (user-only)
- Select in **Settings → Desktop → Appearance → Icons**

**Fonts**: COSMIC supports custom system fonts via `.config` changes or settings.

### Using Community Themes

Popular theme sources:
- [cosmic-themes.org](https://cosmic-themes.org/)
- [catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop)
- [rose-pine/cosmic-desktop](https://github.com/rose-pine/cosmic-desktop)

Installation steps:
1. Download/clone theme (e.g., `~/cosmic-themes/catppuccin`)
2. In **Settings → Desktop → Appearance** click **Import**
3. Select the theme's `cosmic-settings.ron` or similar
4. For terminal colors: **COSMIC Terminal → View → Color schemes… → Import** and select the theme's `cosmic-term.ron`

---

## Changing Wallpaper

### GUI: Per-Display, Fit, Slideshow

Access via:
- **Desktop right-click → Change Wallpaper**, or
- **Settings → Wallpaper**

Options:
- Choose **per-display** wallpapers or single image for all displays
- Set **fit mode** (Fill, Fit, etc.)
- Enable **slideshow** toggle

### Programmatic Wallpaper: Edit CosmicBackground

When `XDG_CURRENT_DESKTOP` is `COSMIC`, the wallpaper path lives in:

```
~/.config/cosmic/com.system76.CosmicBackground/v1/all
```

RON structure:

```ron
(
    // ...
    source: Path("/full/path/to/current/wallpaper.jpg"),
    // ...
)
```

**Minimal Bash example**:

```bash
#!/usr/bin/env bash
# set-cosmic-wallpaper /path/to/image

set -euo pipefail

IMG="$(realpath "$1")"
CFG="$HOME/.config/cosmic/com.system76.CosmicBackground/v1/all"

cp "$CFG" "$CFG.bak.$(date +%s)"

# Update the RON field in-place
sed -r --in-place \
  's,source: Path\(".*"\),source: Path("'"$IMG"'"),' \
  "$CFG"
```

**Notes**:
- Value **must** be an absolute path
- COSMIC's settings daemon watches this file — changes apply without logging out
- Keep RON syntax intact (comma placement, parentheses); only swap the string

---

## Dock & Panel: GUI Controls

COSMIC ships one top **Panel** and one bottom **Dock** by default, both managed through the same settings.

Open **Settings → Desktop → Desktop & Panels** for each:

### Behavior / Position
- Auto-hide on/off
- Screen edge (top / bottom / left / right) and which display(s)
- Dock-specific toggle to completely disable it

### Style
- Gap between panel/dock and screen edges
- "Extend to screen edges" vs centered, shorter bar
- Match system appearance or force dark/light
- Size slider (small ↔ large)
- Background opacity

### Applets / Contents
- "Configure panel applets" to add/remove/reorder applets in the Panel or Dock

---

## Dock & Panel: Programmatic Tweaks (RON Files)

### Panel & Dock Config Directory

COSMIC panel configuration is stored in:

```
~/.config/cosmic/com.system76.CosmicPanel.Panel/
```

With versioned subdirs like `v1/`. Files are small RON values controlling layout, sizes, and applet positions.

The Dock is implemented by `cosmic-panel` as well, with different fields handling panel vs dock sections.

### Example: Move Applets to the Center Segment

Create/overwrite:

```
~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_center
```

Content:

```ron
Some([
    "com.system76.CosmicAppletTime",
    "com.system76.CosmicAppletNotifications",
])
```

Takes effect immediately in the running session.

### Example: Programmatically Shrink Panel/Dock Size

**Center segment size** (`~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/size_center`):

```ron
Some(XS)
```

**Wing sizes** (`~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/size_wings`):

```ron
Some((Some(XS), Some(S)))
```

`PanelSize` options: `XS`, `S`, `M`, `L`, `XL`

**Script example**:

```bash
CFG_DIR="$HOME/.config/cosmic/com.system76.CosmicPanel.Panel/v1"

cat > "$CFG_DIR/size_center" <<'EOF'
Some(XS)
EOF

cat > "$CFG_DIR/size_wings" <<'EOF'
Some((Some(XS), Some(XS)))
EOF
```

### Syncing Dock/Panel Layout Across Machines

All configuration lives in `~/.config/cosmic`, making it dotfile-friendly:

1. Configure COSMIC via GUI on one machine
2. Copy or git-track:
   ```bash
   ~/.config/cosmic/com.system76.CosmicPanel.Panel
   ~/.config/cosmic/com.system76.CosmicBackground
   ~/.config/cosmic/com.system76.CosmicTheme.*
   ```
3. On new machine (after logging into COSMIC once), drop directories into `~/.config/cosmic/` and log out/in if needed

COSMIC devs confirm syncing `.config/cosmic` is the intended way to copy your setup.

---

## Programmatically Querying / Reacting to Theme

Read current dark/light mode from:

```
~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark
```

Contains `true` or `false`.

**Shell example**:

```bash
if [ "$(tr -d ' \n' < ~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark)" = "true" ]; then
  echo "COSMIC is in dark mode"
else
  echo "COSMIC is in light mode"
fi
```

**Pro tip**: Use a **systemd user `.path` unit** to watch that file and run a script whenever it changes, keeping your editor/terminal/apps in sync with COSMIC's theme.

---

## Practical Workflow Suggestion

Given how fast COSMIC is evolving, safest workflow:

1. Use **Settings** to get the desktop / theme / dock roughly where you want it
2. Inspect `~/.config/cosmic` to see exactly which tiny RON files changed
3. Script **only** those fields you care about (wallpaper, panel sizes, plugins, theme mode), backing up the originals first
4. Version the whole `~/.config/cosmic` subtree in git for full reproducibility

---

## References

1. [Helix Theme Syncing with COSMIC](https://matthewsanabria.dev/posts/helix-theme-syncing-with-cosmic/) — Matthew Sanabria
2. [COSMIC - Official NixOS Wiki](https://wiki.nixos.org/wiki/COSMIC)
3. [Reconsider configuration folder structure](https://github.com/pop-os/cosmic-epoch/discussions/1245) — cosmic-epoch Discussion #1245
4. [rose-pine/cosmic-desktop](https://github.com/rose-pine/cosmic-desktop) — Soho vibes for COSMIC Desktop
5. [Customizing COSMIC: Theming and Applications](https://blog.system76.com/post/customizing-cosmic-theming-and-applications/) — System76 Blog
6. [Pop!_OS 24.04 COSMIC Alpha](https://itsfoss.com/news/pop-os-24-04-cosmic-alpha/) — It's FOSS
7. [Custom icon themes for Cosmic](https://www.reddit.com/r/pop_os/comments/1ovvuzx/is_it_possible_to_createinstall_custom_icon/) — Reddit
8. [COSMIC theming](https://system76.com/cosmic/theming) — System76
9. [COSMIC Themes](https://cosmic-themes.org/) — Community theme repository
10. [catppuccin/cosmic-desktop](https://github.com/catppuccin/cosmic-desktop) — Soothing pastel theme
11. [Variety set_wallpaper script](https://raw.githubusercontent.com/varietywalls/variety/refs/heads/master/data/scripts/set_wallpaper) — GitHub
12. [Panel and Dock configuration design](https://github.com/pop-os/cosmic-epoch/issues/102) — cosmic-epoch Issue #102
13. [COSMIC Windows 10 theme and layout](https://www.reddit.com/r/pop_os/comments/1k11opw/cosmic_windows_10_theme_and_layout/) — Reddit
14. [Config File for Cosmic](https://github.com/pop-os/cosmic-epoch/issues/216) — cosmic-epoch Issue #216





