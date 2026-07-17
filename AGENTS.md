# Repository Guidelines

Conventions for working in this repo. It is a single-user setup project for Pop!_OS 24 /
COSMIC, not a general library — read [docs/design.md](docs/design.md) before making structural
changes, so you preserve the "one orchestrator" shape on purpose.

## Project Layout

```
install.sh          Bootstrap: prereqs + chezmoi binary + `chezmoi init --apply`.
cosmikase.yaml      The manifest. Only keys that code actually reads belong here.
Makefile            Dev conveniences (help, setup, install, apply, preflight, theme,
                    lint, test, plugins, plugins-install, clean).
bin/                Runtime bash scripts (cosmikase menu, theme trio, update, databases,
                    power-helper, preflight, dropterm, wallpapers) + cosmikase-lib.sh
                    (sourced) + cosmikase-chezmoi (a PEP 723 `uv run --script` helper).
chezmoi/            chezmoi source dir. Dotfiles as dot_* files/templates, COSMIC UX under
                    dot_config/cosmic/, and run_onchange_/run_after_ scripts that read the
                    manifest and install packages. `.chezmoi.toml.tmpl` is the single source
                    of the default theme (nord).
themes/             One dir per theme: curated per-app config files (cursor.json, cosmic.ron,
                    cosmic-term.ron, ghostty.conf, …) copied into place; wallpapers.yaml is a
                    fetch manifest for wallpapers with a known upstream source.
plugins/            Cargo workspace: plugin-common crate + five pop-launcher plugins.
docs/               design.md, cli-reference.md, configuration-reference.md, keybindings.md,
                    troubleshooting.md, pop-launcher-plugins.md, private-tools.md, and six
                    researched guides.
tests/              pytest (cosmikase-chezmoi + surviving scripts) + container smoke test.
```

## Build, Test, and Development Commands

Always go through the Makefile or `install.sh`, so behavior stays reproducible:

- `./install.sh` — bootstrap and apply on a real machine (`--dry-run` to preview).
- `make apply` — `chezmoi apply` only (re-render dotfiles, no bootstrap).
- `make preflight` — run `bin/cosmikase-preflight` (hardware/environment gate).
- `make lint` — `shellcheck` on `bin/*` + `install.sh`, `ruff` on the Python files.
- `make test` — the pytest suite (via `uv run`).
- `make plugins` / `make plugins-install` — build / install the Cargo workspace plugins.

The chezmoi source directory is `chezmoi/`. Locally you can point chezmoi at it with
`chezmoi --source ./chezmoi ...`.

## Coding Style

- **Shell** is the default. Every script starts with `#!/usr/bin/env bash` and
  `set -euo pipefail`, uses 4-space indentation, and must pass `shellcheck`.
- **Guard every action** — check before you act (`command -v`, `dpkg -s`, `flatpak info`) so
  scripts are idempotent and safe to re-run.
- Reuse helpers from `bin/cosmikase-lib.sh` (`log`, `notify`, `require_theme`, `find_helper`,
  history helpers) instead of reimplementing them.
- **Python** is limited to one PEP 723 single-file `uv` script (`bin/cosmikase-chezmoi`);
  keep it lint-clean under `ruff`. Do not reintroduce an installable Python package.
- `snake_case` for functions and variables; `UPPER_SNAKE_CASE` for constants and env vars.

## Security & Secrets

- **Never commit secrets or private identifiers.** No content from `~/.ssh/config`, `~/.aws`,
  `~/.gitconfig` identity, and no private hostnames. Templates use placeholders and chezmoi
  template guards (e.g. `{{ if lookPath "agent-balance" }}`).
- Out-of-repo private tooling is documented — by name and restore step, never with secrets —
  in [docs/private-tools.md](docs/private-tools.md).
- Keep `.env.example` sanitized and up to date.

## Documentation

- Pair every "do X" with "undo / restore / verify X" (backups need restore steps; config
  changes need rollback notes).
- Call out platform-specific limits and safety rails (mount checks before writing, fallback
  users for auth changes, Timeshift/Firejail scope).
- Keep the six researched guides intact; prune only references to deleted machinery.

## Commits & PRs

- Imperative mood, summary under ~72 chars, short body for rationale.
- Keep PRs focused; avoid unrelated refactors. Note what changed and how it was tested.
