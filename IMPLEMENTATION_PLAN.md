# Implementation Plan: Omarchy-style Pop!_OS 24 with Ghostty

Goals: keep COSMIC hotkeys/workflow on Pop!_OS 24; use apt + Flatpak only; default to Ghostty; per-component install toggles via YAML; Omarchy-like tools/themes; optional HP ZBook Ultra notes.

## Repository layout
- `install.sh` – main idempotent installer reading YAML config.
- `omarchy-pop.yaml` – per-component toggle file (apt, Flatpak, fonts, Ghostty, themes, HP notes).
- `bin/omarchy-pop-theme` – applies theme fragments to Ghostty, kitty fallback, Neovim, btop, starship.
- `bin/omarchy-pop-fonts` – installs JetBrainsMono Nerd Font into `~/.local/share/fonts`.
- `dotfiles/` – base configs for ghostty/kitty, nvim (LazyVim-friendly), btop, starship, shell rc snippets.
- `themes/<name>/` – palette files: `ghostty.conf`, `kitty.conf`, `nvim.lua`, `btop.theme`, `starship.toml`.
- `README.md` – usage + HP notes; `AGENTS.md` – contributor guide.

## Config file (per-component, YAML)
Example `omarchy-pop.yaml`:
```yaml
defaults:
  install: true
  ghostty: true
  theme: osaka-jade

apt:
  core:
    - name: fzf
      desc: fuzzy finder
      install: true
    - name: zoxide
      install: true
    - name: ripgrep
      install: true
    - name: fd-find
      install: true   # binary is fdfind
    - name: bat
    - name: btop
    - name: tmux
    - name: git
    - name: git-lfs
    - name: neovim
    - name: curl
    - name: wget
    - name: jq
    - name: build-essential
  gui:
    - name: pinta
    - name: xournalpp
    - name: fonts-jetbrains-mono
    - name: steam
      install: false   # gaming opt-in
  terminal:
    - name: ghostty
      source: deb      # installer handles fetch from upstream .deb
    - name: kitty
      install: true    # fallback if ghostty disabled or fails

flatpak:
  utility:
    - id: md.obsidian.Obsidian
      install: true
    - id: org.localsend.localsend_app
    - id: com.github.tchx84.Flatseal
    - id: com.spotify.Client
      install: false

fonts:
  nerd:
    - name: JetBrainsMono Nerd Font
      url: https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip
      install: true

themes:
  default: osaka-jade
  available: [osaka-jade, catppuccin]
  paths:
    base: ~/.local/share/omarchy-pop/themes

hp_zbook_ultra:
  emit_notes: true
  oem_kernel: linux-oem-24.04b
  warn_on_mix: true
```

## Installer behavior (`install.sh`)
- Dependencies: `python3`, `python3-yaml` (install via apt if missing) to parse YAML; `rsync`, `curl`, `unzip`, `flatpak`, `fc-cache`. Guard all installs with existence checks.
- Flow:
  1) Source YAML into bash-friendly structures via a small python helper that outputs lists (e.g., JSON -> `jq`-less parsing using python `json` module). Fail fast if config missing.
  2) Pre-flight: `sudo apt update && sudo apt full-upgrade -y`, `sudo fwupdmgr get-updates`, optional `pop-upgrade recovery upgrade from-release` (gated by a flag).
  3) For each apt item with `install: true`, check with `dpkg -s` and install if absent. Honor group fields only for readability; decisions are per item.
  4) Ghostty: if enabled and not installed, download latest `.deb` from ghostty.dev release URL to `/tmp`, `sudo dpkg -i`; if it fails and `kitty.install` is true, install kitty as fallback. Set `TERMINAL=ghostty` in shell snippet when Ghostty succeeds.
  5) Flatpaks: ensure flathub remote; install items with `install: true` and not already present (`flatpak list`).
  6) Fonts: run `bin/omarchy-pop-fonts` using URL from YAML; skip if matching font already in `~/.local/share/fonts`.
  7) Dotfiles: `rsync -av --backup --suffix=.omarchy-pop.bak dotfiles/ ~/.config/` and append shell snippet to `~/.bashrc`/`~/.zshrc` to add `~/omarchy-pop/bin` to PATH and export `TERMINAL`.
  8) Theme: call `omarchy-pop-theme <default>` from YAML; ensure themes staged to `~/.local/share/omarchy-pop/themes`.
  9) Post: print summary and reminders to log out/in for fonts.

## Theme applicator (`bin/omarchy-pop-theme`)
- Input: theme name, themes base path (default `~/.local/share/omarchy-pop/themes`).
- For each component, if file exists in theme dir, copy into place: Ghostty (`~/.config/ghostty/themes/active.conf` and ensure include in `config`), kitty fallback (`~/.config/kitty/theme.conf`), starship (`~/.config/starship.toml`), btop (`~/.config/btop/themes/<name>.theme` + set `color_theme`), Neovim (`~/.config/nvim/lua/omarchy_pop/theme.lua`). No COSMIC theme changes.

## HP ZBook Ultra handling
- Do not auto-install OEM kernel. Instead, emit guidance if `hp_zbook_ultra.emit_notes` is true: webcam/suspend may require Ubuntu `linux-oem-24.04b`; mixing with Pop kernel is unsupported—suggest Ubuntu OEM partition if needed. Mention optional kernel params `amd_iommu=off pcie_aspm=off` for troubleshooting only.

## Optional Ansible
- Mirror YAML into Ansible vars or load YAML directly; tasks for apt items, flatpaks, fonts (non-root), dotfiles, theme apply. Keep Ghostty install as a task that fetches `.deb`.

## Next steps
- Add `omarchy-pop.yaml` with the structure above.
- Implement `install.sh` to parse YAML via python and act per item.
- Add `bin/omarchy-pop-theme` and `bin/omarchy-pop-fonts`.
- Stage starter themes (osaka-jade, catppuccin) and matching dotfiles for Ghostty/LazyVim/btop/starship.
- Document usage and HP notes in `README.md`.
