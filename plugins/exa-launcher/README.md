# Exa Launcher Plugin

A Pop!_OS launcher plugin that provides AI-powered web search via [Exa.ai](https://exa.ai/).

## Prerequisites

- Rust toolchain (`rustup` recommended)
- An Exa.ai API key (sign up at https://exa.ai/)

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
cargo build --release -p exa-launcher
mkdir -p ~/.local/share/pop-launcher/plugins/exa
cp target/release/exa-launcher ~/.local/share/pop-launcher/plugins/exa/
cp exa-launcher/plugin.ron ~/.local/share/pop-launcher/plugins/exa/
```

## Configuration

Set your Exa API key using one of these methods:

### Option 1: Environment variable (recommended)

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
export EXA_API_KEY="your-api-key-here"
```

### Option 2: Config file

Create `~/.config/exa-launcher/config.toml`:

```toml
api_key = "your-api-key-here"
num_results = 8
```

## Usage

1. Open Pop Launcher with `Super` key
2. Type `exa ` followed by your search query
3. Press Enter to open a result in your browser

### Context Menu

Right-click (or use context key) on a result for additional options:
- **Open in browser** - Opens the URL in your default browser
- **Copy URL to clipboard** - Copies the URL to clipboard

## Implementation notes

- **TODO (dependency weight):** this plugin uses `reqwest` (blocking) for a
  single JSON `POST`, which pulls in the full hyper + native-tls (openssl)
  stack. A future pass should swap it for a lighter HTTP client (e.g. `ureq`
  with rustls) to shrink the build and drop the system openssl dependency. This
  was deliberately left unchanged during the plugins-workspace refactor to keep
  that change scoped and reviewable.
- The shared launcher protocol (request/response types, the stdin/stdout
  dispatch loop, clipboard helpers) lives in the `plugin-common` crate.

## Troubleshooting

### Plugin not showing up

- Verify the plugin files exist in `~/.local/share/pop-launcher/plugins/exa/`
- Restart Pop Shell or log out and back in
- Check if other plugins work to ensure pop-launcher is running

### No results / API errors

- Verify your API key is set correctly
- Check your Exa.ai account for API usage limits
- Look for error messages in the search results

### Debug logging

Logs are written to `~/.local/state/exa-launcher.log` (if logging is enabled in Pop Shell settings).





