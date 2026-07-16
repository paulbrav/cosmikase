# Clipboard History Pop Launcher Plugin

A Pop!_OS launcher plugin that provides access to clipboard history via [cliphist](https://github.com/sentriz/cliphist) integration.

## Prerequisites

- Rust toolchain (`rustup` recommended)
- `cliphist` clipboard manager running
- `wl-clipboard` for Wayland clipboard access

## Installation

### Install cliphist

cliphist is installed via Go:

```bash
go install go.senan.xyz/cliphist@latest
```

### Set up cliphist to run on login

Add to your startup applications or shell profile:

```bash
# For Wayland (add to ~/.config/autostart/ or systemd user service)
wl-paste --watch cliphist store
```

### Install the plugin

From the project root:

```bash
make plugins-install
```

This builds every plugin in the `plugins/` Cargo workspace and copies each
binary plus its `plugin.ron` into `~/.local/share/pop-launcher/plugins/<name>/`.

### Manual installation:

The plugins live in one Cargo workspace, so binaries land in the shared
`plugins/target/` directory:

```bash
cd plugins
cargo build --release -p clip-launcher
mkdir -p ~/.local/share/pop-launcher/plugins/clip
cp target/release/clip-launcher ~/.local/share/pop-launcher/plugins/clip/
cp clip-launcher/plugin.ron ~/.local/share/pop-launcher/plugins/clip/
```

## Usage

1. Open Pop Launcher with `Super` key
2. Type `clip ` to see clipboard history (or `clip <query>` to filter)
3. Press Enter to paste the selected entry to your clipboard

### Examples

- `clip ` - Show all clipboard history
- `clip http` - Filter for entries containing "http"
- `clip password` - Search for password-related entries (be careful!)

### Context Menu

Right-click (or use context key) on a result for additional options:
- **Paste to clipboard** - Copies the entry to your current clipboard
- **Delete from history** - Removes the entry from clipboard history

## How It Works

1. `cliphist` runs in the background watching your clipboard via `wl-paste --watch`
2. When you copy something, cliphist stores it in `~/.cache/cliphist/`
3. This plugin queries cliphist to list stored entries
4. When you select an entry, it uses `cliphist decode | wl-copy` to restore it

## Troubleshooting

### Plugin not showing up

- Verify the plugin files exist in `~/.local/share/pop-launcher/plugins/clip/`
- Restart Pop Shell or log out and back in
- Check if other plugins work to ensure pop-launcher is running

### "cliphist not found"

- Ensure cliphist is installed: `go install go.senan.xyz/cliphist@latest`
- Verify it's in your PATH: `which cliphist`
- You may need to add `~/go/bin` to your PATH

### Clipboard empty

- Verify cliphist is running: `pgrep -f "cliphist store"`
- Start cliphist watcher: `wl-paste --watch cliphist store &`
- Copy something and try again

### Images not working

- Image support depends on your Wayland compositor
- Ensure `wl-clipboard` is installed: `sudo apt install wl-clipboard`

## Security Notes

- Clipboard history may contain sensitive data (passwords, etc.)
- Consider setting up cliphist to ignore certain applications
- The history is stored in `~/.cache/cliphist/` - consider encrypting your home directory
- Use "Delete from history" for sensitive entries

## Autostart Configuration

Create a systemd user service for cliphist:

```ini
# ~/.config/systemd/user/cliphist.service
[Unit]
Description=Clipboard history manager
After=graphical-session.target

[Service]
ExecStart=/bin/sh -c 'wl-paste --watch cliphist store'
Restart=always

[Install]
WantedBy=graphical-session.target
```

Enable it:

```bash
systemctl --user enable --now cliphist
```

## License

MIT

