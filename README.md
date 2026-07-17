# Cosmikase — COSMIC Omakase for Pop!_OS

> Rebuild my daily-driver environment on Pop!_OS 24 / COSMIC in one command, the moment my
> hardware (HP ZBook Ultra G1a) is fully supported — with my workflows expressed the
> COSMIC-native way.

Cosmikase is a single-user, single-machine setup repo. One orchestrator (**chezmoi**) applies
the dotfiles and installs everything the manifest (`cosmikase.yaml`) declares. There is no
Ansible, no Python package, and no hidden second orchestration layer — see
[docs/design.md](docs/design.md) for the first-principles rationale.

## Quick Start

```bash
git clone https://github.com/paulbrav/cosmikase ~/Repos/cosmikase
cd ~/Repos/cosmikase

# Bootstrap: install prereqs + chezmoi, then apply dotfiles and packages.
./install.sh

# Preview everything without changing the system first:
./install.sh --dry-run

# Then confirm the hardware is ready for the COSMIC-native workflow:
bin/cosmikase-preflight
```

`install.sh` checks the OS, installs `curl` / `git` / `python3-yaml`, drops the `chezmoi`
binary into `~/.local/bin`, and runs `chezmoi init --source ./chezmoi --apply`. chezmoi's
`run_onchange_` scripts read `cosmikase.yaml` and install apt/flatpak packages, language
runtimes, and CLI tools — each step guarded so re-runs are idempotent.

## Hardware Gate — HP ZBook Ultra G1a

This machine is the reference target. `bin/cosmikase-preflight` prints a PASS/FAIL/WARN table
so you know whether the environment is ready before committing to it: COSMIC session, kernel
support, webcam firmware/driver, fingerprint (fprintd + Synaptics), the power udev rule,
Flathub reachability, and disk space.

The one blocker worth knowing up front is the **webcam**:

- Sensor **OV05C10** behind the **AMD ISP4** block, driven by the `amd_capture` kernel module.
- Requires firmware `isp_4_1_1.bin` under `/lib/firmware/amdgpu/` — no libcamera needed.
- The driver merged into **mainline Linux 7.2**. Pop!_OS 24.04 ships 6.17.9, so until 7.2
  lands you need an interim path: the Ubuntu **OEM kernel**, a **DKMS backport**, or a
  self-built **≥ 7.2** kernel.

Until the webcam driver is generally available, preflight will WARN — the rest of the
environment still installs and runs.

## Structure

```
cosmikase/
├── install.sh          # bootstrap: prereqs + chezmoi + apply
├── cosmikase.yaml      # THE manifest — the only keys code reads
├── Makefile            # dev conveniences (lint, test, plugins, …)
├── bin/                # runtime bash scripts + one PEP 723 uv helper
├── chezmoi/            # dotfiles + COSMIC UX + run_onchange package scripts
├── themes/             # one dir per theme: curated per-app config files + wallpaper manifest
├── plugins/            # Cargo workspace: shared crate + 5 pop-launcher plugins
├── docs/               # design + reference + researched guides
└── tests/              # pytest + container smoke test of install.sh
```

## Terminals

Two terminals, each with a clear job:

- **cosmic-term** — the daily driver, themed from each theme's curated `cosmic-term.ron`.
- **Ghostty** — the drop-down quick terminal, toggled with `Super + grave` via
  `bin/cosmikase-dropterm` (Ghostty's own `quick-terminal` is the primary path).

Kitty and Alacritty are not used anywhere in this repo.

## Themes

Each theme is a directory of **curated per-app config files** — `cursor.json`, `cosmic.ron`,
`cosmic-term.ron`, `ghostty.conf`, `btop.theme`, `neovim.lua`, and so on. The theme scripts
copy these files into place directly; there is no intermediate palette layer to keep in sync.

Wallpapers are **not committed to git**. `themes/wallpapers.yaml` records each file's source
URL and sha256; `bin/cosmikase-wallpapers fetch [theme]` downloads and verifies them into
`~/.local/share/cosmikase/backgrounds/`. The default theme is **nord**, defined in exactly one
place: `chezmoi/.chezmoi.toml.tmpl`.

```bash
cosmikase-theme tokyo-night   # switch theme (updates chezmoi + running apps)
cosmikase                     # interactive menu (gum), if installed
```

See [themes/README.md](themes/README.md) for the full roster and per-theme details.

## Pop Launcher Plugins

A single Cargo workspace under `plugins/` (shared `plugin-common` crate + five plugins).
Build and install all of them with `make plugins-install`.

- **bw** (`bw-launcher`) — search your Bitwarden vault; copy password / username / TOTP.
- **exa** (`exa-launcher`) — AI-powered web search via [Exa.ai](https://exa.ai/).
- **ssh** (`ssh-launcher`) — connect to hosts from `~/.ssh/config`.
- **man** (`man-launcher`) — fuzzy-search and open man pages.
- **clip** (`clip-launcher`) — browse and paste clipboard history via cliphist.

Details in [docs/pop-launcher-plugins.md](docs/pop-launcher-plugins.md).

## Testing

```bash
make lint            # shellcheck bin/* install.sh + ruff on the python files
make test            # pytest suite (cosmikase-chezmoi + surviving bash scripts)
./tests/container-smoke.sh   # build an Ubuntu 24.04 image and run install.sh --ci
```

The container smoke test answers "can this build a machine?" without touching your system:
it runs `install.sh` in CI mode (`COSMIKASE_CI=1` skips flatpak/GUI-only steps) and asserts
chezmoi applied, apt core packages installed, and preflight exits cleanly (WARN, not crash,
with no COSMIC/webcam present).

For full-desktop manual verification, run a Pop!_OS 24 COSMIC ISO in a VM with
[quickemu](https://github.com/quickemu-project/quickemu): snapshot before `./install.sh`, run
it, then verify preflight, a theme switch, and the COSMIC shortcuts.

## Documentation

- [Design](docs/design.md) — why the repo is shaped the way it is.
- [CLI Reference](docs/cli-reference.md) — every surviving command.
- [Configuration Reference](docs/configuration-reference.md) — the `cosmikase.yaml` schema.
- [Keybindings](docs/keybindings.md) · [Troubleshooting](docs/troubleshooting.md)
- [Pop Launcher Plugins](docs/pop-launcher-plugins.md) · [Private Tools](docs/private-tools.md)

**Researched guides:** [COSMIC Theming](docs/cosmic-theming.md) ·
[Editor Theming](docs/editor-theming.md) · [Zellij](docs/zellij.md) ·
[YubiKey Setup](docs/yubikey-setup.md) · [Browser Sandboxing](docs/firejail-browsers.md) ·
[Backup Strategy](docs/backup-strategy.md)

## Uninstall

```bash
chezmoi purge          # remove chezmoi-managed dotfiles
rm -rf ~/.local/share/cosmikase   # themes, wallpapers, helper data
```

Packages are not removed automatically; remove any you no longer want with `apt`/`flatpak`.

## License

MIT — see [LICENSE](LICENSE).
