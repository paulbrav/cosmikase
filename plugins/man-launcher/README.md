# Man Pages Pop Launcher Plugin

A Pop!_OS launcher plugin that provides quick access to man pages using `apropos` for fuzzy searching.

## Prerequisites

- Rust toolchain (`rustup` recommended)
- `man-db` package (usually pre-installed on Linux)
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
cargo build --release -p man-launcher
mkdir -p ~/.local/share/pop-launcher/plugins/man
cp target/release/man-launcher ~/.local/share/pop-launcher/plugins/man/
cp man-launcher/plugin.ron ~/.local/share/pop-launcher/plugins/man/
```

## Usage

1. Open Pop Launcher with `Super` key
2. Type `man ` followed by your search query
3. Press Enter to open the man page in a terminal

### Examples

- `man ls` - Search for man pages matching "ls"
- `man printf` - Search for printf (shows both shell and C library versions)
- `man network` - Search for network-related man pages
- `man config` - Search for configuration-related man pages

### Context Menu

Right-click (or use context key) on a result for additional options:
- **Open in terminal** - Opens the man page in your terminal
- **Open in browser** - Opens the man page on man.cx
- **Copy man command** - Copies `man <section> <name>` to clipboard

## Man Page Sections

The plugin searches these sections:
- **1** - User commands
- **2** - System calls
- **3** - Library functions
- **5** - File formats and conventions
- **7** - Miscellaneous
- **8** - System administration commands

## Terminal Priority

The plugin tries terminals in this order (kitty and alacritty were dropped;
the project standardizes on cosmic-term + ghostty):
1. cosmic-term (COSMIC default)
2. Ghostty (binary)
3. Ghostty (snap)
4. GNOME Terminal
5. xterm

The man command is passed as separate arguments (`man <section> <name>`), so no
shell interprets the page name.

## Troubleshooting

### Plugin not showing up

- Verify the plugin files exist in `~/.local/share/pop-launcher/plugins/man/`
- Restart Pop Shell or log out and back in
- Check if other plugins work to ensure pop-launcher is running

### No results found

- Ensure the man-db package is installed: `sudo apt install man-db`
- Update the man page database: `sudo mandb`
- Try a broader search term

### "apropos: nothing appropriate" errors

This usually means the man page database needs to be built:

```bash
sudo mandb
```

## License

MIT

