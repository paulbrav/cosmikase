# Exa Launcher Plugin

A Pop!_OS launcher plugin that provides AI-powered web search via [Exa.ai](https://exa.ai/).

## Prerequisites

- Rust toolchain (`rustup` recommended)
- An Exa.ai API key (sign up at https://exa.ai/)

## Installation

### From the project root:

```bash
make exa-install
```

This will:
1. Build the plugin with `cargo build --release`
2. Copy the binary and config to `~/.local/share/pop-launcher/plugins/exa/`

### Manual installation:

```bash
cd plugins/exa-launcher
cargo build --release
mkdir -p ~/.local/share/pop-launcher/plugins/exa
cp target/release/exa-launcher ~/.local/share/pop-launcher/plugins/exa/
cp plugin.ron ~/.local/share/pop-launcher/plugins/exa/
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





