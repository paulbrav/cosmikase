# SSH Hosts Pop Launcher Plugin

A Pop!_OS launcher plugin that provides quick access to SSH hosts defined in your `~/.ssh/config`.

## Prerequisites

- Rust toolchain (`rustup` recommended)
- SSH config file at `~/.ssh/config`
- A terminal emulator (cosmic-term, Ghostty, or GNOME Terminal)

## Installation

### From the project root:

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
cargo build --release -p ssh-launcher
mkdir -p ~/.local/share/pop-launcher/plugins/ssh
cp target/release/ssh-launcher ~/.local/share/pop-launcher/plugins/ssh/
cp ssh-launcher/plugin.ron ~/.local/share/pop-launcher/plugins/ssh/
```

## Usage

1. Open Pop Launcher with `Super` key
2. Type `ssh ` followed by your search query (or just `ssh ` to see all hosts)
3. Press Enter to connect to a host

### Examples

- `ssh ` - List all hosts from your SSH config
- `ssh prod` - Search for hosts matching "prod"
- `ssh dev` - Search for hosts matching "dev"

### Context Menu

Right-click (or use context key) on a result for additional options:
- **Connect via SSH** - Opens a terminal and connects
- **Copy SSH command** - Copies `ssh hostname` to clipboard
- **Copy hostname** - Copies the hostname/IP to clipboard

## SSH Config Format

The plugin parses standard SSH config format:

```
Host myserver
    HostName 192.168.1.100
    User admin
    Port 2222

Host production
    HostName prod.example.com
    User deploy

Host *
    # Wildcard hosts are ignored
```

**Note**: Wildcard patterns (`*`, `?`) are filtered out.

## Terminal Priority

The plugin tries terminals in this order (kitty and alacritty were dropped;
the project standardizes on cosmic-term + ghostty):
1. cosmic-term (COSMIC default)
2. Ghostty (binary)
3. Ghostty (snap)
4. GNOME Terminal
5. xterm

The host alias is passed as a separate argument (`ssh <name>`), so no shell
interprets it.

## Troubleshooting

### Plugin not showing up

- Verify the plugin files exist in `~/.local/share/pop-launcher/plugins/ssh/`
- Restart Pop Shell or log out and back in
- Check if other plugins work to ensure pop-launcher is running

### No hosts found

- Verify your SSH config exists at `~/.ssh/config`
- Check that hosts are defined with `Host` directives
- Ensure host names don't contain wildcard characters

### Terminal not opening

- Verify you have at least one supported terminal installed
- Check that the terminal is in your PATH

## License

MIT

