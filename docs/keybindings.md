# Keybinding Reference

Complete reference for keyboard shortcuts across the Cosmikase stack, organized by the layer that captures each key combination.

## Keybinding Hierarchy

Keys are processed in layers. The first layer to match a shortcut consumes it:

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: COSMIC Desktop (captures Super+* shortcuts)       │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Terminal Emulator (Ghostty/Kitty/Alacritty)        │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Zellij (Normal mode captures Ctrl+g/t/r/s/p/m/o)   │
│          (Locked mode passes all except Ctrl+g)             │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Shell / FZF (Ctrl+r, Ctrl+t, Alt+c)                │
├─────────────────────────────────────────────────────────────┤
│ Layer 5: Application (nvim, less, etc.)                     │
└─────────────────────────────────────────────────────────────┘
```

## Layer 1: COSMIC Desktop

COSMIC uses **Super** key combinations for desktop actions. These are captured before reaching any application.

| Shortcut | Action |
|----------|--------|
| `Super` | Open launcher |
| `Super + A` | Open applications menu |
| `Super + W` | Open workspaces view |
| `Super + T` | Open terminal |
| `Super + F` | Open Files |
| `Super + Q` | Close window |
| `Super + M` | Maximize/restore window |
| `Super + Tab` | Switch applications |
| `Super + `` ` | Switch windows of current app |
| `Super + ←/→` | Snap window left/right |
| `Super + Ctrl + ←/→` | Snap window left/right half |
| `Super + ↑/↓` | Navigate between panes |
| `Super + Shift + ↑/↓` | Move window between workspaces |
| `Super + Ctrl + ↑/↓` | Navigate workspaces |
| `Super + Home/End` | First/last workspace |
| `Super + V` | Toggle notifications |
| `Super + D` | Toggle workspace menu |
| `Super + P` | Cycle display layout |
| `Super + Esc` | Lock screen |

### Tiling-Specific

| Shortcut | Action |
|----------|--------|
| `Super + S` | Toggle stacking |
| `Super + O` | Toggle orientation |
| `Super + G` | Float/unfloat window |
| `Super + Enter` | Enter window adjustment mode |

**Note**: COSMIC does NOT capture `Alt+1-9`, `Alt+t`, `Alt+n`, or `Alt+hjkl` — these pass through to applications.

---

## Layer 2: Terminal Emulators

Terminal emulators (Ghostty, Kitty, Alacritty) have minimal keybindings that don't conflict with Zellij.

### Ghostty

| Shortcut | Action |
|----------|--------|
| `Shift + Insert` | Paste from clipboard |
| `Ctrl + Insert` | Copy to clipboard |

### Kitty

| Shortcut | Action |
|----------|--------|
| `Ctrl + Shift + C` | Copy |
| `Ctrl + Shift + V` | Paste |

---

## Layer 3: Zellij Terminal Multiplexer

Zellij is configured to start in **Locked mode** by default. This allows shell shortcuts (FZF's Ctrl+r/t) to work normally. Press `Ctrl+g` to enter Normal mode for Zellij navigation.

### Locked Mode (Default)

| Shortcut | Action |
|----------|--------|
| `Ctrl + g` | Switch to Normal mode |

All other keys pass through to the shell/application.

### Normal Mode

Press `Ctrl+g` from Locked mode to enter. Navigation and mode switching available.

**Navigation (Alt-based, don't conflict with COSMIC):**

| Shortcut | Action |
|----------|--------|
| `Alt + h` / `Alt + ←` | Move focus left |
| `Alt + l` / `Alt + →` | Move focus right |
| `Alt + j` / `Alt + ↓` | Move focus down |
| `Alt + k` / `Alt + ↑` | Move focus up |
| `Alt + 1-9` | Go to tab 1-9 |
| `Alt + t` | New tab |
| `Alt + n` | New pane |
| `Alt + d` | Detach session |
| `Alt + f` | Toggle floating panes |

**Mode Switching (Ctrl-based):**

| Shortcut | Action |
|----------|--------|
| `Ctrl + g` | Return to Locked mode |
| `Ctrl + p` | Enter Pane mode |
| `Ctrl + t` | Enter Tab mode |
| `Ctrl + r` | Enter Resize mode |
| `Ctrl + s` | Enter Scroll mode |
| `Ctrl + o` | Enter Session mode |
| `Ctrl + m` | Enter Move mode |

**Note**: In Normal mode, `Ctrl+t`, `Ctrl+r` are captured by Zellij. Use Locked mode if you need FZF shortcuts.

### Sub-Modes (Pane, Tab, Resize, etc.)

See [Zellij documentation](zellij.md) for detailed keybindings in each sub-mode.

---

## Layer 4: Shell / FZF

These work when Zellij is in **Locked mode** (the default).

| Shortcut | Action |
|----------|--------|
| `Ctrl + r` | FZF history search |
| `Ctrl + t` | FZF file search |
| `Alt + c` | FZF directory navigation (cd) |

---

## Layer 5: Cursor / VS Code

These are application-level keybindings.

| Shortcut | Action |
|----------|--------|
| `Ctrl + Shift + I` | Open Composer Agent mode |
| `Ctrl + Shift + P` | Command Palette |
| `Ctrl + Shift + X` | Extensions |
| `Ctrl + P` | Quick Open |
| `Ctrl + `` ` | Toggle terminal |

---

## Known Conflicts and Trade-offs

### Ctrl+g Reserved by Zellij

`Ctrl+g` is used by Zellij for mode switching in both Locked and Normal modes. Applications inside Zellij cannot receive `Ctrl+g`.

**Affected**:
- Vim/Neovim: `Ctrl+g` shows file info (use `:file` instead)
- Emacs: `Ctrl+g` cancels commands
- less: `Ctrl+g` prints current file position

**Workaround**: These are typically non-critical bindings with alternatives. If you heavily rely on `Ctrl+g`, consider using a different Zellij mode switch key.

### FZF vs Zellij Normal Mode

When Zellij is in Normal mode:
- `Ctrl+r` enters Resize mode (not FZF history)
- `Ctrl+t` enters Tab mode (not FZF file search)

**Solution**: Stay in Locked mode (the default) for shell work. Press `Ctrl+g` only when you need Zellij navigation, then `Ctrl+g` again to return to Locked mode.

---

## Quick Reference Card

```
COSMIC (Super key)
├── Super+T → Terminal
├── Super+Q → Close window
├── Super+Tab → App switcher
└── Super+W → Workspaces

Zellij (from Locked mode)
├── Ctrl+g → Enter Normal mode
│   └── Normal mode
│       ├── Alt+hjkl → Navigate panes
│       ├── Alt+1-9 → Switch tabs
│       ├── Alt+t → New tab
│       ├── Alt+n → New pane
│       └── Ctrl+g → Back to Locked
└── (Locked passes keys to shell)

Shell/FZF (in Zellij Locked mode)
├── Ctrl+r → History search
├── Ctrl+t → File search
└── Alt+c → Directory jump

Cursor/VS Code
├── Ctrl+Shift+I → Composer Agent
└── Ctrl+Shift+P → Command Palette
```

---

## See Also

- [Zellij Terminal Multiplexer](zellij.md) — Detailed Zellij keybindings and configuration
- [COSMIC Desktop Theming](cosmic-theming.md) — Desktop appearance and settings
- [Pop!_OS Keyboard Shortcuts](https://support.system76.com/articles/pop-keyboard-shortcuts/) — Official System76 reference

