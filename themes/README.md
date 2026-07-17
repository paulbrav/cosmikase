# Cosmikase Themes

A theme collection for Pop!_OS / COSMIC. Each theme is a directory of curated,
per-app config files; the theme scripts copy the relevant file into place for
each app. There is no intermediate palette representation — a theme's checked-in
files are exactly what the runtime applies. Wallpapers with a known upstream
source live in a fetch manifest rather than in git.

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
| `cursor.json` | Cursor / VS Code / Antigravity colour theme name, extension id, and colours |
| `ghostty.conf` | Ghostty terminal colours |
| `cosmic-term.ron` | COSMIC Terminal colour scheme |
| `cosmic.ron` | COSMIC desktop colour theme |
| `neovim.lua` | Neovim colourscheme snippet |
| `btop.theme` | btop resource-monitor theme |
| `opencode.json` | OpenCode colour theme |
| `antigravity.conf` | Antigravity launcher colours |
| `backgrounds/` | Wallpapers still tracked in git (see Wallpapers) |
| `preview.png` | Theme preview image (ported themes) |
| `light.mode` | Present only for light themes — marks COSMIC light mode |

Terminals are **cosmic-term** (daily driver) and **ghostty** (quick-terminal
dropdown). Kitty and Alacritty were removed, so their per-theme configs are gone.
Hyprland/waybar/mako/walker/swayosd/starship/chromium/icons theme files were also
removed — cosmikase targets COSMIC, not a Hyprland stack.

## Colours and the dark/light flag

Each app's colours live directly in that app's config file (`cursor.json`,
`cosmic-term.ron`, `ghostty.conf`, …); editing a theme means editing those files.
`cursor.json`'s `light` flag is the declared dark/light truth for a theme; light
themes also carry a `light.mode` marker file, which flips COSMIC into light mode.

## Why there is no per-theme manifest

Earlier revisions carried a `theme.yaml` manifest whose only consumer was the
(now deleted) Python package, and later a `palette.yaml` layer that nothing at
runtime read. Both were removed. `cursor.json` carries the editor theme metadata,
each per-app file carries its own colours, and the default wallpaper is simply the
first image found in `backgrounds/` — so nothing reads a per-theme manifest any more.

## Wallpapers

[`wallpapers.yaml`](wallpapers.yaml) is a fetch manifest: it lists only the
wallpapers with a known upstream source, each as `filename` + `url` + `sha256`.
To keep the repo small those files were removed from git and are re-fetched
(sha256-verified) on demand. Wallpapers without a verifiable upstream source are
not listed — they ship in-tree under `themes/<name>/backgrounds/`, where git owns
their integrity.

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
  upstream and are kept in-tree.
- **catppuccin / osaka-jade** — custom (Catppuccin palette art, Unsplash /
  Wallhaven imagery); kept in-tree.
- **cosmic-dark / cosmic-light** — space and abstract imagery; kept in-tree
  (provenance unverified).

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

1. Create `themes/<name>/` and add `cursor.json` (editor theme name + extension +
   colours). For a light theme, set its `light` flag and add a `light.mode`
   marker file next to it.
2. Add the per-app config files by copying an existing theme's and editing the
   colours: `ghostty.conf`, `cosmic-term.ron`, `cosmic.ron`, `neovim.lua`,
   `btop.theme`, `opencode.json`, `antigravity.conf`.
3. Add wallpapers under `backgrounds/`. If a wallpaper has a byte-verifiable
   upstream source, add a `filename` + `url` + `sha256` record to
   `wallpapers.yaml` and drop the file from git; otherwise leave it in-tree.

## Credits

- Ported themes: [basecamp/omarchy](https://github.com/basecamp/omarchy)
- Catppuccin: [catppuccin/catppuccin](https://github.com/catppuccin/catppuccin)
- Nord: [nordtheme/nord](https://github.com/nordtheme/nord)
- Gruvbox: [morhetz/gruvbox](https://github.com/morhetz/gruvbox)
- Tokyo Night: [enkia/tokyo-night-vscode-theme](https://github.com/enkia/tokyo-night-vscode-theme)
- Rosé Pine: [rose-pine/rose-pine-theme](https://github.com/rose-pine/rose-pine-theme)
- Pop!_OS wallpapers: [pop-os/wallpapers](https://github.com/pop-os/wallpapers) (CC BY-SA 4.0)
