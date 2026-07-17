# Configuration Reference

`cosmikase.yaml` is the manifest — the single declarative list of what gets installed. It is
read at apply time by chezmoi's `run_onchange_` scripts (plain `python3` + `yaml`), so those
scripts rerun whenever the manifest changes.

**Guiding rule:** the manifest carries **only keys that code reads**. There are no decorative
`method`/`url`/`args`/`check` fields, no version pins that contradict the installer, and no
`default` that nothing consumes. If a field is here, a script acts on it. This file describes
the shape; `cosmikase.yaml` in the repo root is the authoritative, current content.

## Top-level structure

```yaml
apt:            # apt packages, grouped
  core:  [...]
  gui:   [...]
  system: [...]
flatpak: [...]        # flatpak application IDs
runtimes: {...}       # language runtimes + their version pins
cargo_tools: [...]    # installed via `cargo install`
go_tools:    [...]    # installed via `go install`
uv_tools:    [...]    # installed via `uv tool install`
npm_globals: [...]    # installed via `npm -g`
ai_tools:    [...]    # AI CLIs (claude, codex, grok, …)
```

The default theme is **not** set here — it lives in exactly one place,
`chezmoi/.chezmoi.toml.tmpl` (a chezmoi prompt with the `nord` default). Hardware
notes for the HP ZBook Ultra G1a are **not** in the manifest either — no script
reads them; see [Troubleshooting](troubleshooting.md#preflight--hardware).

---

## apt

APT packages, split into logical groups. Each item has a `name` and an optional `install`
flag (default: enabled). Set `install: false` to opt a package out.

```yaml
apt:
  core:
    - name: fzf
    - name: ripgrep
    - name: fd-find
      alias: fdfind        # optional: command name when it differs from the package
  gui:
    - name: xournalpp
  system:
    - name: v4l-utils      # webcam tooling
    - name: ydotool        # input automation
    - name: docker.io
      install: false       # opt-out example
```

- `name` (required) — the apt package name.
- `install` (optional, default true) — set `false` to skip.
- `alias` (optional) — command name when it differs (e.g. `fd-find` → `fdfind`).

The package script installs each enabled apt group, checking with `dpkg -s` before acting.

---

## flatpak

A flat list of Flatpak application IDs (reverse-DNS notation). The package script installs
`flatpak` and adds the Flathub remote first if any app is enabled.

```yaml
flatpak:
  - md.obsidian.Obsidian
  - org.signal.Signal
  - com.system76.KeyboardConfigurator
```

(Skipped entirely when `COSMIKASE_CI=1`.)

---

## runtimes

Language runtimes with their pinned versions. The runtimes script installs each via its
official installer, guarded by `command -v`.

```yaml
runtimes:
  rust:  { via: rustup }
  node:  { via: nvm, version: "lts" }
  go:    { version: "1.24.4" }
  # bun, uv, julia (juliaup), zig, …
```

Version pins live **here**, next to the installer that reads them — not scattered across the
docs.

---

## Tool lists (cargo_tools, go_tools, uv_tools, npm_globals, ai_tools)

Each is a list of items installed by the tools script, one ecosystem each. Every installer is
guarded (`command -v`) so re-runs are idempotent.

```yaml
cargo_tools:
  - cargo-sweep
go_tools:
  - go.senan.xyz/cliphist@latest
uv_tools:
  - ruff
  - yt-dlp
  - marimo
npm_globals:
  - "@bitwarden/cli"
  - "@mermaid-js/mermaid-cli"
  - "@earendil-works/pi-coding-agent"
ai_tools:
  - name: claude        # native installer
  - name: codex         # via npm (the canonical entry — no duplicate)
  - name: grok
```

Items may be plain strings or `{ name: ..., install: ... }` where a per-item toggle is useful.

---

## Validating the manifest

```bash
# YAML is well-formed
python3 -c "import yaml; yaml.safe_load(open('cosmikase.yaml'))"

# Preview the whole apply (packages included) without changing anything
./install.sh --dry-run
```

## Best Practices

1. Keep `cosmikase.yaml` in version control.
2. Comment your choices with `#`.
3. Keep related packages in the same apt group.
4. Mark truly optional apt packages `install: false`.
5. Add version pins next to the runtime they configure, not in prose.

## See Also

- [CLI Reference](cli-reference.md) — the commands that consume this file.
- [Design](design.md) — why the manifest is shaped this way.
- [Troubleshooting Guide](troubleshooting.md)
