# Zellij Terminal Multiplexer

Complete guide to using Zellij in the cosmikase setup.

## Overview

This repository supports both **tmux** and **Zellij** as terminal multiplexers. Zellij is installed via APT (if enabled in `cosmikase.yaml`) and configured via chezmoi with a modal keybinding approach inspired by Omakub.

## Installation

Zellij is installed automatically if enabled in your configuration:

```yaml
apt:
  core:
    - name: zellij
      install: true
```

Verify installation:

```bash
zellij --version
```

## Configuration Location

After `chezmoi apply`, the generated config is at:

```bash
~/.config/zellij/config.kdl
```

Layouts live at:

```bash
~/.config/zellij/layouts/
```

The configuration template is managed by chezmoi at:

```bash
chezmoi/dot_config/zellij/config.kdl.tmpl
```

## Usage

### Starting Zellij

```bash
# Start a new session
zellij

# Start with a specific layout
zellij --layout compact

# Attach to existing session
zellij attach

# List sessions
zellij list-sessions
```

### Basic Concepts

- **Sessions**: Top-level container (like tmux sessions)
- **Tabs**: Within a session, like tmux windows
- **Panes**: Within tabs, like tmux panes
- **Modes**: Different keybinding contexts (Normal, Pane, Tab, etc.)

## Keybinding System

This configuration uses a **modal approach** where Zellij starts in **Locked** mode to prevent accidental key presses. Press `Ctrl+g` to unlock and enter Normal mode.

### Mode Overview

| Mode | Purpose | Entry Key |
|------|---------|-----------|
| **Locked** | Default mode, prevents accidental input | Starts here, or `Ctrl+g` from Normal |
| **Normal** | Main navigation mode | `Ctrl+g` from Locked |
| **Pane** | Pane management | `Ctrl+p` from Normal |
| **Tab** | Tab management | `Ctrl+t` from Normal |
| **Resize** | Resize panes | `Ctrl+r` from Normal |
| **Scroll** | Scroll through pane output | `Ctrl+s` from Normal |
| **Session** | Session management | `Ctrl+o` from Normal |
| **Move** | Move panes | `Ctrl+m` from Normal |

### Locked Mode

Default mode when Zellij starts. Prevents accidental key presses.

**Keybindings:**
- `Ctrl+g`: Switch to Normal mode

### Normal Mode

Main navigation and control mode.

**Navigation:**
- `Alt+h` / `Alt+Left`: Move focus left (pane or tab)
- `Alt+l` / `Alt+Right`: Move focus right (pane or tab)
- `Alt+j` / `Alt+Down`: Move focus down (pane)
- `Alt+k` / `Alt+Up`: Move focus up (pane)

**Panes:**
- `Alt+n`: New pane
- `Alt+d`: Detach from session
- `Alt+f`: Toggle floating panes

**Tabs:**
- `Alt+t`: New tab
- `Alt+1` through `Alt+9`: Go to tab 1-9

**Mode Switching:**
- `Ctrl+p`: Enter Pane mode
- `Ctrl+t`: Enter Tab mode
- `Ctrl+r`: Enter Resize mode
- `Ctrl+s`: Enter Scroll mode
- `Ctrl+o`: Enter Session mode
- `Ctrl+m`: Enter Move mode
- `Ctrl+g`: Return to Locked mode

### Pane Mode

Manage panes within the current tab.

**Navigation:**
- `h` / `Left`: Move focus left
- `l` / `Right`: Move focus right
- `j` / `Down`: Move focus down
- `k` / `Up`: Move focus up
- `p`: Switch focus

**Pane Operations:**
- `n`: New pane (horizontal split)
- `d`: New pane below (vertical split)
- `r`: New pane to the right (horizontal split)
- `x`: Close focused pane
- `f`: Toggle fullscreen
- `z`: Toggle pane frames
- `w`: Toggle floating panes
- `e`: Toggle pane embed/floating
- `c`: Rename pane

**Exit:**
- `Ctrl+p`: Return to Normal mode

### Tab Mode

Manage tabs within the current session.

**Navigation:**
- `h` / `Left` / `k` / `Up`: Go to previous tab
- `l` / `Right` / `j` / `Down`: Go to next tab
- `1`-`9`: Go to tab 1-9

**Tab Operations:**
- `n`: New tab
- `x`: Close current tab
- `r`: Rename tab
- `s`: Toggle active sync tab
- `Tab`: Toggle tab bar

**Exit:**
- `Ctrl+t`: Return to Normal mode

### Resize Mode

Resize panes using vim-style keys.

**Resize:**
- `h` / `Left`: Increase left
- `j` / `Down`: Increase down
- `k` / `Up`: Increase up
- `l` / `Right`: Increase right
- `H`: Decrease left
- `J`: Decrease down
- `K`: Decrease up
- `L`: Decrease right
- `=` / `+`: Increase all
- `-`: Decrease all

**Exit:**
- `Ctrl+r`: Return to Normal mode

### Scroll Mode

Scroll through pane output.

**Scrolling:**
- `j` / `Down`: Scroll down
- `k` / `Up`: Scroll up
- `Ctrl+f` / `PageDown` / `l` / `Right`: Page down
- `Ctrl+b` / `PageUp` / `h` / `Left`: Page up
- `d`: Half page down
- `u`: Half page up
- `Ctrl+c`: Scroll to bottom

**Search:**
- `s`: Enter search mode
- `e`: Edit scrollback

**Exit:**
- `Ctrl+s`: Return to Normal mode

### Session Mode

Manage Zellij sessions.

**Operations:**
- `d`: Detach from session
- `w`: Launch session manager (floating)
- `Ctrl+s`: Enter Scroll mode

**Exit:**
- `Ctrl+o`: Return to Normal mode

### Move Mode

Move panes within tabs.

**Movement:**
- `h` / `Left`: Move pane left
- `j` / `Down`: Move pane down
- `k` / `Up`: Move pane up
- `l` / `Right`: Move pane right
- `n` / `Tab`: Move pane forward
- `p`: Move pane backward

**Exit:**
- `Ctrl+m`: Return to Normal mode

## Configuration Options

### UI Settings

```kdl
pane_frames false        // Hide pane borders
mouse_mode true          // Enable mouse support
copy_command "xclip -selection clipboard"  // Clipboard command
copy_on_select true      // Copy on selection
```

### Theme

The theme is automatically set based on your active cosmikase theme:

```kdl
theme "nord"  // Set via chezmoi template
```

### Default Layout

```kdl
default_layout "compact"  // Use compact layout by default
```

## Layouts

Zellij supports custom layouts defined in `~/.config/zellij/layouts/`.

### Creating a Custom Layout

Create a file in `~/.config/zellij/layouts/`:

```kdl
// ~/.config/zellij/layouts/my-layout.kdl
default_tab_template {
    pane size=1 borderless=true {
        plugin location="zellij:tab-bar"
    }
    children
    pane size=2 borderless=true {
        plugin location="zellij:status-bar"
    }
}

tab name="main" {
    pane
    pane split_direction="vertical" {
        pane
        pane
    }
}
```

Use the layout:

```bash
zellij --layout my-layout
```

## Session Management

### Detaching

```bash
# Detach from session (Alt+d in Normal mode, or d in Session mode)
# Or use Ctrl+d (if configured)
```

### Attaching

```bash
# List sessions
zellij list-sessions

# Attach to session
zellij attach <session-name>

# Attach to most recent
zellij attach
```

### Killing Sessions

```bash
# Kill a specific session
zellij kill-session <session-name>

# Kill all sessions
zellij kill-all-sessions
```

## Customization

### Editing Configuration

The configuration is managed by chezmoi. To customize:

1. **Edit the template:**
   ```bash
   chezmoi edit ~/.config/zellij/config.kdl
   ```

2. **Or edit the source:**
   ```bash
   nano chezmoi/dot_config/zellij/config.kdl.tmpl
   ```

3. **Apply changes:**
   ```bash
   chezmoi apply
   ```

### Adding Keybindings

Add keybindings in the appropriate mode block:

```kdl
normal {
    bind "Ctrl x" { NewPane; }  // Add custom binding
}
```

### Changing Copy Command

For Wayland, update the copy command:

```kdl
copy_command "wl-copy"  // Wayland clipboard
```

For X11:

```kdl
copy_command "xclip -selection clipboard"  // X11 clipboard
```

## Comparison with tmux

If you're familiar with tmux, here's a quick comparison:

| tmux | Zellij | Notes |
|------|--------|-------|
| Session | Session | Same concept |
| Window | Tab | Same concept |
| Pane | Pane | Same concept |
| `C-b` prefix | Modal modes | Zellij uses modes instead of prefix |
| `C-b d` detach | `Alt+d` or `d` in Session mode | Different key |
| `C-b c` new window | `Alt+t` new tab | Different key |
| `C-b %` split vertical | `Alt+n` new pane | Different key |
| `C-b "` split horizontal | `d` in Pane mode | Different key |

**tmux config location:**
```bash
~/.config/tmux/tmux.conf
```

## Tips and Tricks

### Quick Pane Creation

1. Enter Pane mode: `Ctrl+p`
2. Create pane: `n` (horizontal) or `d` (vertical) or `r` (right)

### Efficient Tab Switching

- Use `Alt+1` through `Alt+9` for quick tab access
- Or `Ctrl+t` then `1`-`9` in Tab mode

### Scrolling

- `Ctrl+s` to enter Scroll mode
- Use vim-style keys (`j`/`k`) to scroll
- `Ctrl+c` to jump to bottom

### Floating Panes

- `Alt+f` in Normal mode to toggle floating
- Or `w` in Pane mode

### Session Persistence

Zellij sessions persist across terminal restarts. Use `zellij attach` to reconnect.

## Troubleshooting

### Keybindings Not Working

1. **Check mode:** Make sure you're in the correct mode
2. **Verify config:** Check `~/.config/zellij/config.kdl` exists
3. **Reload config:** Restart Zellij or run `chezmoi apply`

### Copy/Paste Issues

1. **Check clipboard command:**
   ```bash
   # Test clipboard command
   echo "test" | xclip -selection clipboard  # X11
   echo "test" | wl-copy  # Wayland
   ```

2. **Update config:**
   ```kdl
   copy_command "wl-copy"  // For Wayland
   ```

### Theme Not Applying

The theme is set via chezmoi template. Update it:

```bash
cosmikase-theme <theme-name>
```

Or manually:

```bash
chezmoi edit ~/.config/zellij/config.kdl
# Change theme line
chezmoi apply
```

## See Also

- [Zellij Documentation](https://zellij.dev/documentation/)
- [Zellij Keybindings Reference](https://zellij.dev/documentation/keybindings)


