# Bitwarden Pop Launcher Plugin

A Pop!_OS launcher plugin that provides quick access to your Bitwarden vault. Search for credentials and copy passwords, usernames, or TOTP codes directly to your clipboard.

## Prerequisites

- Rust toolchain (`rustup` recommended)
- [Bitwarden CLI](https://bitwarden.com/help/cli/) (`bw`) installed and in PATH
- GNOME Keyring or compatible Secret Service for session storage

## Installation

### From the project root:

```bash
make bw-install
```

This will:
1. Build the plugin with `cargo build --release`
2. Copy the binary and config to `~/.local/share/pop-launcher/plugins/bw/`

### Manual installation:

```bash
cd plugins/bw-launcher
cargo build --release
mkdir -p ~/.local/share/pop-launcher/plugins/bw
cp target/release/bw-launcher ~/.local/share/pop-launcher/plugins/bw/
cp plugin.ron ~/.local/share/pop-launcher/plugins/bw/
```

## Session Management

The plugin needs an active Bitwarden session to access your vault. There are two ways to provide this:

### Option 1: Environment variable

Unlock your vault and export the session:

```bash
export BW_SESSION=$(bw unlock --raw)
```

Add to your shell profile to persist across terminal sessions.

### Option 2: System keyring (recommended)

Store your session in the system keyring for automatic access:

```bash
# Unlock vault and get session
BW_SESSION=$(bw unlock --raw)

# Store in keyring using secret-tool (GNOME)
echo -n "$BW_SESSION" | secret-tool store --label="Bitwarden Session" session bw-launcher
```

The plugin will automatically retrieve the session from the keyring.

### Helper script

You can create a helper script `bw-session-store`:

```bash
#!/bin/bash
# Store Bitwarden session in keyring

if ! command -v bw &> /dev/null; then
    echo "Error: Bitwarden CLI (bw) not found"
    exit 1
fi

# Check login status
if ! bw login --check &> /dev/null; then
    echo "Not logged in. Please run: bw login"
    exit 1
fi

# Unlock and store session
echo "Unlocking Bitwarden vault..."
SESSION=$(bw unlock --raw)

if [ -n "$SESSION" ]; then
    echo -n "$SESSION" | secret-tool store --label="Bitwarden Session" session bw-launcher
    echo "Session stored in keyring successfully!"
else
    echo "Failed to unlock vault"
    exit 1
fi
```

## Usage

1. **Setup**: Ensure Bitwarden CLI is logged in and session is available (see above)
2. **Search**: Open Pop Launcher (`Super` key) and type `bw ` followed by your search query
3. **Copy password**: Press `Enter` on a result to copy its password
4. **Context menu**: Right-click (or press context key) for more options:
   - Copy password
   - Copy username
   - Copy TOTP code (if available)

### Examples

- `bw github` - Search for GitHub credentials
- `bw bank` - Search for banking logins
- `bw email` - Search for email accounts

## Troubleshooting

### Plugin not showing up

- Verify the plugin files exist in `~/.local/share/pop-launcher/plugins/bw/`
- Restart Pop Shell: `killall pop-shell` or log out and back in
- Check if other plugins work to ensure pop-launcher is running

### "Vault locked" error

- Run `bw unlock` in a terminal and store the session (see Session Management above)
- Verify Bitwarden CLI is installed: `bw --version`
- Check login status: `bw login --check`

### "Session expired" error

- Your session has expired or been invalidated
- Run `bw unlock` again and update the stored session

### No results found

- Ensure you're searching with enough characters
- Check if the item exists in your vault: `bw list items --search "query"`

### Clipboard not working

- For Wayland: ensure `wl-copy` is installed (`wl-clipboard` package)
- For X11: ensure `xclip` is installed

## Security Notes

- The session key grants full access to your vault - treat it as sensitive
- Sessions are stored in the system keyring which is encrypted at rest
- Consider using a shorter session timeout in Bitwarden settings
- The plugin never stores or logs your master password

## License

MIT

