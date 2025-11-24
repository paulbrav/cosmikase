# Omarchy-style Pop!_OS setup

Pop!_OS 24 target with COSMIC hotkeys intact, apt + Flatpak only, Ghostty as default terminal, Omarchy-inspired tools/themes, and per-item install toggles (CLI/TUI, Flatpaks including Brave/Bitwarden/ProtonVPN, runtimes, AI tools).

## Quick start
1. Review `omarchy-pop.yaml` and flip `install: true/false` per item (apt, Flatpak, Ghostty, fonts, themes, runtimes, AI tools). Set `deb_url` on AI tools if you have installers; otherwise they are skipped with a note.
2. Run the installer from the repo root:
   ```bash
   ./install.sh
   ```
   Use `CONFIG_FILE=/path/to/config.yaml ./install.sh` to point at a custom config.
3. Switch themes anytime:
   ```bash
   omarchy-pop-theme osaka-jade   # or catppuccin
   theme-tui                      # Textual picker installed via uv
   ```

## Firmware and recovery updates
- Controlled via `defaults.run_fw_update` and `defaults.run_recovery_upgrade` in `omarchy-pop.yaml`.
- Runs after core apt packages so `fwupd`/`pop-upgrade` can be installed first.
- If `fwupdmgr` or `pop-upgrade` are missing, the installer logs a skip message instead of failing.

## What the installer does
- Uses `omarchy-pop.yaml` to drive per-component installs (apt + Flatpak including Brave/Bitwarden/ProtonVPN), runtimes (rust/bun/uv/julia), uv tool installs (`ruff`, `pyrefly` by default), and optional AI tool installers (manual by default; set URLs to automate).
- Prefers Ghostty via upstream `.deb` (set `GHOSTTY_DEB_URL` to override); falls back to kitty if Ghostty disabled or fails.
- Installs JetBrainsMono Nerd via `omarchy-pop-fonts` into `~/.local/share/fonts`.
- Copies dotfiles to `~/.config` and themes to `~/.local/share/omarchy-pop/themes`; seeds a default `~/.bashrc` if you don't have one.
- Appends PATH/TERMINAL and sources `~/.config/shell/omarchy-pop.sh` in `~/.bashrc` / `~/.zshrc`.

## Theme system
- Theme fragments live in `themes/<name>/` and cover multiple terminal emulators (Alacritty, Ghostty, kitty), Wayland components (Waybar, Mako, Hyprland), development tools (Neovim, btop, starship), and system integration (cursors, icons, browser).
- **15 Complete Themes Available:**
  - **Your Custom Themes:** `pop-default`, `catppuccin`, `osaka-jade`
  - **Official Omarchy Themes:** `tokyo-night`, `nord`, `gruvbox`, `kanagawa`, `everforest`, `rose-pine`, `catppuccin-latte`, `matte-black`, `ristretto`, `ethereal`, `flexoki-light`, `hackerman`
- Each theme includes wallpapers in `backgrounds/` directories
- Apply with `omarchy-pop-theme <name>`; default comes from `defaults.theme` in YAML.
- Alternatively, run `theme-tui` (Textual TUI installed via `uv tool install`) to browse and apply themes.
- See `themes/README.md` for complete theme documentation and usage instructions.

## HP ZBook Ultra G1a notes
- Pop ships its own kernel; OEM Ubuntu kernel `linux-oem-24.04b` (needed for webcam on Ubuntu) is **not** installed automatically.
- If webcam/suspend fail on Pop, consider an Ubuntu 24.04 OEM partition for the vendor kernel instead of mixing kernels into Pop.
- Optional troubleshooting (manual): kernel params `amd_iommu=off pcie_aspm=off` in `/etc/default/grub` if you hit dock/suspend quirks.

## Testing
- For a quick container smoke test (Ubuntu 24 base, no Ghostty/Flatpak/fonts/runtimes): `./tests/container-smoke.sh` (set `ENGINE=docker` if needed).
- For full-stack validation, run inside a Pop!_OS 24 VM, edit `omarchy-pop.yaml`, and rerun `./install.sh`; switch themes with `omarchy-pop-theme <name>`.

## Documentation
- [Running Brave or Firefox inside Firejail](docs/firejail-browsers.md) - Step-by-step guide for sandboxing browsers on Pop!_OS/Ubuntu.
- [Pop!_OS Backup Strategy](docs/backup-strategy.md) - Comprehensive guide for rsync scripts, Timeshift snapshots, and restic backups.
- [YubiKey Setup Guide](docs/yubikey-setup.md) - Instructions for local PAM (sudo/polkit) integration and SSH 2FA with FIDO2 keys.
