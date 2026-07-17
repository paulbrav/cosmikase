# Private / out-of-repo tools

Cosmikase deliberately does **not** vendor the owner's personal binaries, scripts,
credentials, or model weights. Several parts of the environment (COSMIC shortcuts,
autostart entries, systemd user units) reference tools that live outside this repo.
Everything that touches them is wrapped in a chezmoi guard
(`{{ "{{" }} if lookPath "toolname" {{ "}}" }} ... {{ "{{" }} end {{ "}}" }}`), so on a
machine where the tool is absent the config simply is not created — nothing breaks.

This page is the honest inventory: what each tool is, roughly where it lives, and one
line on how to restore it. **No secrets, hostnames, IPs, or credentials appear here or
anywhere in the repo.**

## How the guards work

- COSMIC custom shortcut `Super+D` (transclip) is only written when `transclip` is on
  `PATH` at `chezmoi apply` time.
- `~/.config/autostart/*.desktop` and `~/.config/systemd/user/*.{service,timer}` for
  these tools render empty (and chezmoi therefore skips them) unless the tool is found.
- The referenced subcommands (`agent-balance balance`, `agent-balance tray`,
  `transclip daemon`, `transclip toggle`) are **placeholders** — verify them against
  your actual CLIs and adjust the templates in `chezmoi/dot_config/`.

## The tools

| Tool | What it is | Where it lives | Restore (one line) |
|------|-----------|----------------|--------------------|
| **agent-pick / agent-balance** | Personal suite that picks and load-balances across multiple Claude Code accounts (reads a local account pool + balancer state). | `~/.local/bin/agent-*`, state under `~/.claude-accounts/` (`.active` pool, `.balancer-state.json`). | Reinstall from your personal agent-pick/agent-balance repo; recreate `~/.claude-accounts/` from your own backups. Never commit credentials. |
| **transclip** | Clipboard-transcription / push-to-talk dictation tool; toggled by `Super+D`. | `~/.local/bin/transclip` (+ its config). | Reinstall from your personal transclip repo. |
| **granite_speach venv** | Python venv hosting the IBM Granite speech ASR model that transclip talks to (the other half of the "pair"). | A dedicated venv, e.g. `~/.venvs/granite_speach/`; optional launcher `transclip-granite` on PATH. | `python -m venv ~/.venvs/granite_speach && pip install <granite-speech deps>`; download the Granite speech weights per their license. |
| **autohand** | Personal desktop-automation helper. | `~/.local/bin/autohand`. | Reinstall from your personal autohand repo. |
| **mise** | Public polyglot dev-tool/version manager (mise-en-place). | `~/.local/bin/mise`, config `~/.config/mise/`. | `curl https://mise.run \| sh` (see <https://mise.jdx.dev/>). |
| **LM Studio** | Public local-LLM desktop app / server. | Installed app (AppImage or `~/.lmstudio/`). | Download from <https://lmstudio.ai/>. |
| **Miniconda** | Public conda distribution. | `~/miniconda3/`. | Installer from <https://docs.conda.io/projects/miniconda/>. |

## Restoring the automation that references them

After the tools are back on `PATH`, re-run `chezmoi apply` so the guarded shortcuts,
autostart entries, and systemd units get written, then enable the timers/services:

```sh
chezmoi apply
systemctl --user daemon-reload
# only the units that now exist:
systemctl --user enable --now agent-balance.timer 2>/dev/null || true
systemctl --user enable --now cargo-sweep.timer   2>/dev/null || true
```

(`cargo-sweep` is a public cargo subcommand and part of the cosmikase manifest, so its
weekly timer is written on any machine that has run the toolchain install — it is listed
here only because it shares the same enable step.)

The autostart `.desktop` entries need no enabling; they run at the next login.
