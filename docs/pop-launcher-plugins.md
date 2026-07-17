# Pop Launcher Plugins

This guide covers the launcher plugins in cosmikase: the built-in plugins that ship with the
launcher and the five custom plugins developed for this project.

On COSMIC, the launcher (`cosmic-launcher`, opened with the `Super` key) uses **pop-launcher**
as its backend, so pop-launcher plugins work under both Pop!_OS's classic launcher and COSMIC.

## Overview

pop-launcher is a modular, IPC-based launcher service from System76. Plugins are standalone
executables that talk to it via JSON over stdin/stdout. Each plugin:

1. Lives in `~/.local/share/pop-launcher/plugins/<name>/`
2. Contains a binary and a `plugin.ron` configuration file
3. Responds to search queries and activation requests
4. Can provide context-menu options

### Workspace layout

The custom plugins live in a single **Cargo workspace** under `plugins/`:

```
plugins/
├── Cargo.toml          # [workspace] members = plugin-common + the five plugins
├── Cargo.lock          # one lockfile for the whole workspace
├── plugin-common/      # shared crate: the Request/Response protocol + stdio loop
├── bw-launcher/
├── exa-launcher/
├── ssh-launcher/
├── man-launcher/
└── clip-launcher/
```

Every plugin depends on `plugin-common = { path = "../plugin-common" }`, so the JSON protocol
lives in exactly one place. Each plugin keeps its own `plugin.ron` and `README`.

### Plugin configuration

Each plugin has a `plugin.ron` file:

```ron
(
    name: "Plugin Name",
    description: "What the plugin does",
    bin: (
        path: "binary-name",
    ),
    icon: Name("icon-name"),
    query: (
        isolate: true,
        regex: "^prefix ",
    )
)
```

The `regex` field determines when the plugin activates — e.g. `"^ssh "` activates when you
type "ssh " followed by a query.

---

## Built-in Plugins

These ship with pop-launcher and are available by default.

### Calculator (`calc`)

**Prefix:** None (just type math expressions)

- `2 + 2` → 4
- `sqrt(144)` → 12
- `100 * 1.15` → 115

### Application Search (`desktop_entries`)

**Prefix:** None (default behavior). Searches installed applications by name and keywords.

### File Search (`find`)

**Prefix:** `find ` — searches for files using the `find` command.

### Recent Files (`recent`)

**Prefix:** `recent ` — shows recently accessed files from GTK recent files.

### Scripts (`scripts`)

**Prefix:** `/` — executes scripts from `~/.local/share/pop-launcher/scripts/`.

```bash
mkdir -p ~/.local/share/pop-launcher/scripts
# Add executable scripts here
```

### Terminal Commands

**Prefix:** `:` — runs commands in a terminal window (e.g. `:htop`).

### Web Search (`web`)

**Prefix:** varies by engine.

| Prefix | Search Engine |
|--------|---------------|
| `g ` | Google |
| `ddg ` | DuckDuckGo |
| `wiki ` | Wikipedia |

### Audio Devices (`pulse`)

**Prefix:** None — switches audio output/input devices.

---

## Custom Plugins

Developed for cosmikase and installed from the Cargo workspace.

**Install all of them at once:**

```bash
make plugins-install
```

This builds the workspace (`cargo build --release`) and copies each binary + `plugin.ron`
into `~/.local/share/pop-launcher/plugins/<name>/`.

### Bitwarden (`bw`)

**Prefix:** `bw ` — search your Bitwarden vault and copy passwords, usernames, or TOTP codes.

**Prerequisites:** Bitwarden CLI (`bw`) installed, logged in, session stored in the keyring.

**Usage:** `bw github`, `bw bank`

**Context menu:** copy password / username / TOTP.

**Session setup:**
```bash
BW_SESSION=$(bw unlock --raw)
echo -n "$BW_SESSION" | secret-tool store --label="Bitwarden Session" session bw-launcher
```

### Exa Search (`exa`)

**Prefix:** `exa ` — AI-powered web search via [Exa.ai](https://exa.ai/).

**Prerequisites:** an Exa.ai API key.

**Configuration:**
```bash
# Option 1: environment variable
export EXA_API_KEY="your-api-key"

# Option 2: config file (~/.config/exa-launcher/config.toml)
api_key = "your-api-key"
num_results = 8
```

**Usage:** `exa machine learning tutorials`

**Context menu:** open in browser / copy URL.

### SSH Hosts (`ssh`)

**Prefix:** `ssh ` — quick-connect to hosts defined in `~/.ssh/config`.

**Prerequisites:** an SSH config at `~/.ssh/config`; a terminal (cosmic-term, Ghostty, or
GNOME Terminal).

**Usage:** `ssh ` (list all), `ssh prod` (filter).

**Context menu:** connect / copy SSH command / copy hostname.

### Man Pages (`man`)

**Prefix:** `man ` — search and open man pages using `apropos`.

**Prerequisites:** `man-db` (pre-installed on most systems).

**Usage:** `man ls`, `man printf`, `man network`

**Context menu:** open in terminal / open in browser (man.cx) / copy man command.

| Section | Description |
|---------|-------------|
| 1 | User commands |
| 2 | System calls |
| 3 | Library functions |
| 5 | File formats |
| 7 | Miscellaneous |
| 8 | System administration |

### Clipboard History (`clip`)

**Prefix:** `clip ` — browse and paste from clipboard history via
[cliphist](https://github.com/sentriz/cliphist).

**Prerequisites:** `cliphist` running + `wl-clipboard` for Wayland.

**Setup:**
```bash
go install go.senan.xyz/cliphist@latest
# systemd user service: ~/.config/systemd/user/cliphist.service
#   ExecStart=/bin/sh -c 'wl-paste --watch cliphist store'
systemctl --user enable --now cliphist
```

**Usage:** `clip ` (history), `clip http` (filter).

**Context menu:** paste / delete from history.

---

## Cleaning Build Artifacts

```bash
make clean
```

---

## Creating a Custom Plugin

To add a plugin to the workspace:

1. **Create the crate:**
```bash
mkdir -p plugins/my-launcher/src
```

2. **Create `plugins/my-launcher/Cargo.toml`** depending on the shared crate:
```toml
[package]
name = "my-launcher"
version = "0.1.0"
edition = "2021"

[dependencies]
plugin-common = { path = "../plugin-common" }
# plugin-specific deps here

[profile.release]
strip = true
lto = true
```

3. **Add it to the workspace members** in `plugins/Cargo.toml`.

4. **Create `plugins/my-launcher/plugin.ron`:**
```ron
(
    name: "My Plugin",
    description: "What it does",
    bin: (
        path: "my-launcher",
    ),
    icon: Name("application-x-executable"),
    query: (
        isolate: true,
        regex: "^my ",
    )
)
```

5. **Implement `src/main.rs`** using `plugin-common` for the protocol. It handles these JSON
   requests via stdin — `{"Search": "query"}`, `{"Activate": id}`, `{"Context": id}`,
   `{"ActivateContext": {"id": id, "context": ctx}}`, `"Exit"` — and emits `{"Append": {...}}`,
   `{"Clear": "Clear"}`, `{"Finished": "Finished"}`, `{"Close": "Close"}`. See the existing
   plugins for complete examples.

6. **Build and install:**
```bash
make plugins-install
```

---

## Troubleshooting

### Plugin not appearing

1. Verify the files exist:
```bash
ls ~/.local/share/pop-launcher/plugins/<name>/
# Should show: <name> (binary) and plugin.ron
```

2. Restart the launcher service so it re-scans plugins. On COSMIC:
```bash
systemctl --user restart cosmic-launcher.service 2>/dev/null \
  || pkill -x pop-launcher
```
The service restarts on demand, or just log out and back in. (Do **not** use
`killall pop-shell` — that is GNOME/Pop-shell, not COSMIC.)

3. Test the binary directly:
```bash
echo '{"Search": "test"}' | ~/.local/share/pop-launcher/plugins/<name>/<name>
```

### Plugin crashes

Run it manually and read stderr:
```bash
cd ~/.local/share/pop-launcher/plugins/<name>
echo '{"Search": "test"}' | ./<name>
```

### Search results not showing

1. Ensure the prefix matches your query.
2. Check prerequisites are installed (cliphist, bw, etc.).
3. Verify any required environment variables are set.

---

## Security Considerations

- **Bitwarden plugin:** the session token is stored in the system keyring, encrypted at rest.
- **Clipboard plugin:** history may contain sensitive data — use "delete from history" for
  sensitive items.
- **SSH plugin:** reads `~/.ssh/config`, which should be mode `600`.

---

## License

All custom plugins are licensed under MIT.
