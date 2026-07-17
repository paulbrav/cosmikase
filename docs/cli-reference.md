# CLI Reference

Every command that ships in cosmikase after the first-principles rebuild. Runtime tooling is
bash scripts in `bin/` plus two single-file `uv` helpers; there is no Python CLI package.

## Table of Contents

- [Bootstrap](#bootstrap)
  - [install.sh](#installsh)
- [Make targets](#make-targets)
- [Runtime scripts](#runtime-scripts)
  - [cosmikase](#cosmikase)
  - [cosmikase-preflight](#cosmikase-preflight)
  - [cosmikase-theme](#cosmikase-theme)
  - [cosmikase-wallpapers](#cosmikase-wallpapers)
  - [cosmikase-dropterm](#cosmikase-dropterm)
  - [cosmikase-update](#cosmikase-update)
  - [cosmikase-databases](#cosmikase-databases)
  - [cosmikase-cursor-extensions](#cosmikase-cursor-extensions)
  - [cosmikase-power-helper](#cosmikase-power-helper)
- [Internal helpers](#internal-helpers)
  - [cosmikase-chezmoi](#cosmikase-chezmoi)
  - [theme helpers](#theme-helpers)

---

## Bootstrap

### install.sh

The one-command entry point. Bootstraps prereqs and chezmoi, then applies everything.

```bash
./install.sh              # full bootstrap + apply
./install.sh --dry-run    # preview changes (passes through to chezmoi)
```

**What it does:**
1. Checks the OS (warns if not Pop!_OS / Ubuntu 24.04+).
2. Installs prereqs (`curl`, `git`, `python3-yaml`) with a single sudo prompt up front.
3. Installs the `chezmoi` binary to `~/.local/bin` if missing.
4. Runs `chezmoi init --source ./chezmoi --apply`, whose `run_onchange_` scripts read
   `cosmikase.yaml` and install apt/flatpak packages, runtimes, and tools.
5. Suggests running `bin/cosmikase-preflight`.

**Environment:**
- `COSMIKASE_CI=1` — CI-safe mode: skips flatpak / snap / GUI-only steps. Used by the
  container smoke test.

---

## Make targets

Thin wrappers so everyone runs the same steps:

| Target | Action |
|--------|--------|
| `make help` | List targets. |
| `make setup` | Install dev/bootstrap dependencies. |
| `make install` | Run `./install.sh`. |
| `make apply` | `chezmoi apply` only (re-render dotfiles). |
| `make preflight` | Run `bin/cosmikase-preflight`. |
| `make theme` | Launch the theme picker. |
| `make lint` | `shellcheck bin/* install.sh` + `ruff` on the Python files. |
| `make test` | Run the pytest suite via `uv run`. |
| `make plugins` | Build the Cargo workspace (`cargo build --release`). |
| `make plugins-install` | Build + install all pop-launcher plugins. |
| `make clean` | Remove build artifacts. |

---

## Runtime scripts

### cosmikase

Interactive menu (built with `gum`, with a plain-prompt fallback).

```bash
cosmikase
```

**Entries:** Preflight, Install/Update system (`install.sh`), Apply dotfiles
(`chezmoi apply`), Theme picker (`cosmikase-theme`), Update everything (`cosmikase-update`),
Databases (`cosmikase-databases`), Quit.

---

### cosmikase-preflight

Hardware/environment gate. Prints a PASS/FAIL/WARN table and exits `0` only if all checks PASS.

```bash
cosmikase-preflight
```

**Checks:** COSMIC session present; kernel ≥ 7.2 **or** the `amd_capture` module available
(`modinfo`); `/lib/firmware/amdgpu/isp_4_1_1.bin*` present; fingerprint (fprintd + Synaptics);
battery/power udev rule active; Flathub reachable; free disk space.

Runs on non-COSMIC hosts too (missing pieces become WARN, not crashes), so it is usable on the
reference hardware today.

---

### cosmikase-theme

Switch the active theme. chezmoi is the single orchestrator: this command validates the
theme, records history, rewrites `[data].theme` (via `cosmikase-chezmoi`), and runs
`chezmoi apply --force`. The live application — COSMIC, terminals, editors, and per-app
assets — is done once by the `run_onchange_after_10-setup-theme` hook, which fires because
the theme changed. Re-selecting the theme that is already active re-runs the helper scripts
directly, since `run_onchange` will not refire.

```bash
cosmikase-theme <theme-name>
cosmikase-theme --rollback
```

**Common flags:** `--rollback` (previous theme from history), `--quiet` / `-q` (suppress
helper output). Theme history lives at `~/.config/cosmikase/theme-history`. The per-app
helpers (`cosmikase-theme-cosmic`, `-cursor`, `-terminal`) remain directly callable.

See [themes/README.md](../themes/README.md) and [editor-theming.md](editor-theming.md).

---

### cosmikase-wallpapers

Fetch and verify the wallpapers whose upstream source left git. Most wallpapers ship in-tree
under `themes/<name>/backgrounds/` (git owns their integrity); this command handles only the
fetchable ones listed in `themes/wallpapers.yaml`, downloading and sha256-verifying them into
`~/.local/share/cosmikase/backgrounds/<theme>/`.

```bash
cosmikase-wallpapers fetch [theme]   # download + sha256-verify (all themes, or one)
cosmikase-wallpapers verify          # check existing files against the manifest
```

Sources and checksums live in `themes/wallpapers.yaml`. The theme apply hook calls `fetch`
automatically when backgrounds are missing (guarded and offline-safe).

---

### cosmikase-dropterm

Toggle the Ghostty drop-down quick terminal. Bound to `Super + grave`; a fallback for when
Ghostty's built-in `quick-terminal` global is unavailable.

```bash
cosmikase-dropterm
```

---

### cosmikase-update

Update installed software across ecosystems.

```bash
cosmikase-update
```

Updates apt and flatpak packages, language runtimes (rust, uv, bun, node, julia, …), global
CLI tools, and checks firmware (`fwupdmgr`, no automatic install). Requires sudo and a network
connection. Missing tools are skipped, not failed.

---

### cosmikase-databases

Start development databases as Docker containers named `cosmikase-*`.

```bash
cosmikase-databases
```

Creates PostgreSQL (5432), MySQL (3306), Redis (6379), and/or MongoDB (27017) with persistent
volumes. Requires `docker` and `gum`.

**Cleanup:**
```bash
docker rm -f cosmikase-postgres cosmikase-mysql cosmikase-redis cosmikase-mongodb
docker volume rm cosmikase-postgres-data cosmikase-mysql-data cosmikase-redis-data cosmikase-mongodb-data
```

---

### cosmikase-cursor-extensions

Manage Cursor/VS Code extensions from a text file (default `~/.config/Cursor/extensions.txt`).

```bash
cosmikase-cursor-extensions export    # write installed extensions to the list
cosmikase-cursor-extensions install   # install everything in the list
cosmikase-cursor-extensions list      # show what would be installed
cosmikase-cursor-extensions diff      # list vs. installed
```

**Options:** `-f FILE` (custom list). **Env:** `EXTENSIONS_FILE`, `CURSOR_CMD`.

---

### cosmikase-power-helper

Switch System76 power profiles based on AC/battery state. Usually invoked by the power udev
rule, but runnable by hand.

```bash
cosmikase-power-helper            # auto-detect and apply
cosmikase-power-helper --ac       # force performance
cosmikase-power-helper --battery  # force battery
```

Requires `system76-power`.

---

## Internal helpers

### cosmikase-chezmoi

A single-file PEP 723 `uv` script (`#!/usr/bin/env -S uv run --script`, dependency: `tomlkit`)
that atomically edits chezmoi's TOML config (tempfile + rename). It replaces the old Python
package's `chezmoi.py`.

```bash
cosmikase-chezmoi set theme <name>   # set the active theme
cosmikase-chezmoi get theme          # read the current theme
```

Typically called by `cosmikase-theme`; direct use is rarely needed. If `uv` is missing it
prints an actionable error.

### theme helpers

`cosmikase-theme` calls three helpers to push a theme into running applications. They are not
meant to be run directly:

- `cosmikase-theme-cosmic` — updates COSMIC desktop theme (RON files).
- `cosmikase-theme-cursor` — updates Cursor/VS Code color theme.
- `cosmikase-theme-terminal` — updates cosmic-term and Ghostty colors.

Shared bash helpers live in `bin/cosmikase-lib.sh` (sourced, not executed).

---

## See Also

- [README.md](../README.md) — project overview.
- [Configuration Reference](configuration-reference.md) — the `cosmikase.yaml` schema.
- [Troubleshooting Guide](troubleshooting.md) — common issues.
