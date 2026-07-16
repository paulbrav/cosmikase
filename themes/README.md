# Cosmikase Themes

A palette-first theme collection for Pop!_OS / COSMIC. Each theme declares its
colours once in `palette.yaml`; the app-specific files are generated from that
palette (for the two files that are fully derivable) or hand-curated (for the
rest). Wallpapers live in a manifest, not in git.

## Available themes

**Custom (built for cosmikase)**

- **pop-default** — Pop!_OS palette, signature orange + teal accents
- **osaka-jade** — cyan/jade green aesthetic
- **catppuccin** — dark Catppuccin Mocha variant

**Ported from [basecamp/omarchy](https://github.com/basecamp/omarchy)**

- **tokyo-night**, **nord**, **gruvbox**, **kanagawa**, **everforest**,
  **rose-pine**, **catppuccin-latte**, **matte-black**, **ristretto**,
  **ethereal**, **flexoki-light**, **hackerman**
- **cosmic-dark**, **cosmic-light** — COSMIC default-style dark/light schemes

The default theme is **nord**, defined in exactly one place:
`chezmoi/.chezmoi.toml.tmpl`.

## Theme file structure

Each `themes/<name>/` directory contains:

| File | Purpose |
|------|---------|
| `palette.yaml` | **Declared colour source of truth** — 7 colour keys + `variant` (see below) |
| `cursor.json` | Cursor / VS Code / Antigravity colour theme name, extension id, and colours |
| `ghostty.conf` | Ghostty terminal colours (generated from palette or curated) |
| `cosmic-term.ron` | COSMIC Terminal colour scheme (generated from palette or curated) |
| `cosmic.ron` | COSMIC desktop colour theme |
| `neovim.lua` | Neovim colourscheme snippet |
| `btop.theme` | btop resource-monitor theme |
| `opencode.json` | OpenCode colour theme |
| `antigravity.conf` | Antigravity launcher palette |
| `backgrounds/` | Wallpapers still tracked in git (see Wallpapers) |
| `preview.png` | Theme preview image (ported themes) |
| `light.mode` | Present only for light themes — marks COSMIC light mode |

Terminals are **cosmic-term** (daily driver) and **ghostty** (quick-terminal
dropdown). Kitty and Alacritty were removed, so their per-theme configs are gone.
Hyprland/waybar/mako/walker/swayosd/starship/chromium/icons theme files were also
removed — cosmikase targets COSMIC, not a Hyprland stack.

## palette.yaml — the colour source of truth

```yaml
name: Nord
variant: dark          # dark | light
colors:
  background: "#2e3440"
  foreground: "#eceff4"
  accent:     "#88c0d0"
  sidebar:    "#242933"
  terminal:   "#2e3440"
  error:      "#bf616a"
  warning:    "#ebcb8b"
```

These 7 keys were extracted from each theme's `cursor.json` `colors` block.
`variant` comes from `cursor.json`'s `light` flag (the declared truth), which is
also mirrored by the `light.mode` marker on light themes.

### Regenerating ghostty.conf / cosmic-term.ron

`ghostty.conf` and `cosmic-term.ron` are the two files whose colours are fully
derivable from the palette. Regenerate them with the single-file renderer:

```bash
themes/render.py --theme nord      # one theme
themes/render.py --all             # every theme
themes/render.py --all --dry-run   # preview without writing
```

`render.py` is a PEP 723 script (`uv run themes/render.py …`, deps: pyyaml,
jinja2) using the templates in `themes/_templates/`. The renderer reproduces the
palette-defined slots exactly (background, foreground, cursor, and the
black/red/green/yellow ANSI colours) and fills the remaining ANSI slots from the
accent. Use it for **new themes** or to fix drift; richer hand-curated palettes
that ship today are left as-is.

### Why there is no `theme.yaml`

Earlier revisions carried a `theme.yaml` manifest whose only consumer was the
(now deleted) Python package. It was removed. `palette.yaml` is the colour source
of truth, `cursor.json` carries the editor theme metadata, and the default
wallpaper is simply the first image found in `backgrounds/` — so nothing reads a
per-theme manifest any more.

## Wallpapers

Wallpapers are described by [`wallpapers.yaml`](wallpapers.yaml), which records
every wallpaper's `sha256`, byte size, and provenance (`source`). To keep the
repo small, wallpapers with a **verified upstream source** were removed from git;
wallpapers whose source is **unknown** stay in the tree (removing them would be
irreversible data loss — they are listed at the top of the manifest).

Restore or verify the removed wallpapers with:

```bash
bin/cosmikase-wallpapers fetch            # download all removed wallpapers
bin/cosmikase-wallpapers fetch nord       # just one theme
bin/cosmikase-wallpapers verify           # sha256-check what's present
bin/cosmikase-wallpapers list             # show state of every wallpaper
```

`fetch` downloads (sha256-verified) into
`${XDG_DATA_HOME:-~/.local/share}/cosmikase/backgrounds/<theme>/` and is
idempotent and offline-safe. The theme `run_after` hook and
`cosmikase-theme-cosmic` call it automatically when a theme's backgrounds are
missing.

### Attribution

- **pop-default** — official Pop!_OS wallpapers by **Kate Hazen** and
  **Nick Nazzaro**, from [pop-os/wallpapers](https://github.com/pop-os/wallpapers),
  licensed **CC BY-SA 4.0**. These are re-fetched, not committed.
- **omarchy-ported themes** — wallpapers originate from
  [basecamp/omarchy](https://github.com/basecamp/omarchy) (MIT). Two are still
  byte-verifiable upstream and are re-fetched; the rest were renamed/changed
  upstream and are kept in-tree as `unknown` provenance.
- **catppuccin / osaka-jade** — custom (Catppuccin palette art, Unsplash /
  Wallhaven imagery); kept in-tree as `unknown`.
- **cosmic-dark / cosmic-light** — space and abstract imagery; kept in-tree as
  `unknown` pending provenance verification.

## Applying a theme

```bash
cosmikase-theme nord            # switch theme (updates chezmoi, reapplies, reloads apps)
cosmikase-theme --rollback      # revert to the previous theme
```

`cosmikase-theme` updates the chezmoi `theme` value (via `bin/cosmikase-chezmoi`),
reapplies dotfiles, and calls the per-app helpers:

- `cosmikase-theme-cosmic` — COSMIC colours, terminal scheme, dark/light mode, wallpaper
- `cosmikase-theme-cursor` — Cursor / VS Code / Antigravity `workbench.colorTheme`
- `cosmikase-theme-terminal` — reloads ghostty (SIGUSR1); cosmic-term auto-reloads

## Adding a new theme

1. Create `themes/<name>/palette.yaml` with the 7 colour keys and a `variant`.
2. Run `themes/render.py --theme <name>` to generate `ghostty.conf` and
   `cosmic-term.ron`.
3. Add `cursor.json` (editor theme name + extension + colours), and optionally
   `cosmic.ron`, `neovim.lua`, `btop.theme`, `opencode.json`, `antigravity.conf`.
4. Add wallpapers under `backgrounds/`, then record them in `wallpapers.yaml`
   (`bin/cosmikase-wallpapers verify` re-checks sha256s).

## Credits

- Ported themes: [basecamp/omarchy](https://github.com/basecamp/omarchy)
- Catppuccin: [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin)
- Nord: [nordtheme/nord](https://github.com/nordtheme/nord)
- Gruvbox: [morhetz/gruvbox](https://github.com/morhetz/gruvbox)
- Tokyo Night: [enkia/tokyo-night-vscode-theme](https://github.com/enkia/tokyo-night-vscode-theme)
- Rosé Pine: [rose-pine/rose-pine-theme](https://github.com/rose-pine/rose-pine-theme)
- Pop!_OS wallpapers: [pop-os/wallpapers](https://github.com/pop-os/wallpapers) (CC BY-SA 4.0)
