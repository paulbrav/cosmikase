# Design — First Principles

_Last revised 2026-07-16._

Cosmikase serves one sentence:

> Rebuild my daily-driver environment on Pop!_OS 24 / COSMIC in one command, the moment my
> hardware (HP ZBook Ultra G1a) is fully supported — with my workflows expressed the
> COSMIC-native way.

One person, one reference machine. Every piece of complexity has to pay rent against that
sentence. This document records the load-bearing decisions so they are not relitigated by
accident.

## Single orchestrator

chezmoi is the only orchestrator. `install.sh` bootstraps prereqs and the chezmoi binary,
then `chezmoi init --apply` does everything: it renders the dotfiles **and** runs
`run_onchange_` scripts that read `cosmikase.yaml` to install apt/flatpak packages, runtimes,
and CLI tools. The previous shape — Makefile → bash installer → six Ansible invocations →
chezmoi + a mostly-dead Python package — was four mutually-recursive layers whose terminal act
was shelling out to chezmoi anyway. Collapsing to one layer keeps ~95% of the value and
removes an entire class of "which layer owns this?" bugs.

## Manifest honesty

`cosmikase.yaml` is the spine: a declarative list with per-item `install:` toggles. It carries
**only keys that code reads** — no decorative `method`/`url`/`args`/`check` metadata, no
version pins that contradict the installer, no `default` that nothing consumes. If a field is
in the manifest, a script acts on it. Runtime version pins live where they are used, in the
`run_onchange_` scripts.

## Palette-first themes

Each theme's real payload is a seven-key color palette (`palette.yaml`: background,
foreground, accent, error, warning, success, cursor). A renderer (`themes/render.py`)
generates the per-app configs from that palette instead of regex-patching existing files, so
every theme stays consistent and adding one is cheap. Wallpapers are fetched from a
checksummed manifest, never committed — they were the single largest contributor to repo bloat.
The default theme (nord) is defined in exactly one place: `chezmoi/.chezmoi.toml.tmpl`.

## COSMIC UX layer

Workflow muscle memory is expressed the COSMIC-native way: RON atom files under
`~/.config/cosmic/` for shortcuts and tiling, chezmoi-managed `systemd --user` units and
autostart entries for automation, all wrapped in template guards so a machine without the
private tools skips them cleanly. Terminals are cosmic-term (daily) and Ghostty (drop-down
quick terminal); Kitty and Alacritty are gone.

## Hardware preflight

The hardware gate is a script, not prose. `bin/cosmikase-preflight` prints a PASS/FAIL/WARN
table (COSMIC session, kernel/webcam-driver, fingerprint, power udev rule, Flathub, disk) and
runs today on non-COSMIC reference hardware with WARNs. It encodes the real blocker — the
webcam driver (`amd_capture`, sensor OV05C10, firmware `isp_4_1_1.bin`) merged in mainline
Linux 7.2, while Pop 24.04 ships 6.17.9 — so "is this machine ready yet?" is answerable in one
command.

## Private-tools policy

Out-of-repo private tooling (the agent-pick/agent-balance suite, transclip, etc.) is never
copied into the repo and never leaks a secret or hostname. It is documented by name and restore
step in `docs/private-tools.md`, and any automation that references it is guarded so public
machines skip it.

## Deliberately deleted (one line each)

- **Ansible** — its useful logic (pins, install commands) was ported into `run_onchange_`
  scripts; the orchestration layer added nothing chezmoi couldn't do.
- **The Python package** (`src/`, TUI, CLI, config loader, RON "validator") — dead facade; a
  name collision even broke the installer. Only the atomic TOML edit survives as
  `bin/cosmikase-chezmoi`.
- **Committed wallpapers and `target/` artifacts** — replaced by a fetch manifest and
  `.gitignore`; they dominated the packed repo.
- **116 Hyprland/wlroots theme files** (waybar/mako/hyprland/… ) — zero consumers on COSMIC.
- **Kitty & Alacritty** — cosmic-term + Ghostty cover the terminal story honestly.
- **`architecture.md`, `development.md`, `cosmikase-menu.md`, `CHANGELOG.md`** — rotted or
  ceremonial docs for a one-author repo; the repo had more prose than code.
