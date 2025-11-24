#!/usr/bin/env bash
set -euo pipefail

# Update script for omarchy-pop

SUDO="sudo"
if [[ "$(id -u)" -eq 0 ]]; then
  SUDO=""
elif ! command -v sudo >/dev/null 2>&1; then
  echo "sudo not found, running without it (may fail)"
  SUDO=""
fi

echo "== Updating System Packages (APT) =="
$SUDO apt update && $SUDO apt upgrade -y
$SUDO apt autoremove -y

if command -v flatpak >/dev/null 2>&1; then
  echo "== Updating Flatpaks =="
  flatpak update -y
fi

if command -v rustup >/dev/null 2>&1; then
  echo "== Updating Rust =="
  rustup update
fi

if command -v uv >/dev/null 2>&1; then
  echo "== Updating uv and tools =="
  uv self update || echo "uv self update failed (maybe installed via package manager?)"
  uv tool upgrade --all
fi

if command -v juliaup >/dev/null 2>&1; then
  echo "== Updating Julia =="
  juliaup update
fi

if command -v npm >/dev/null 2>&1; then
  echo "== Updating Global NPM Packages =="
  # Some setups might require sudo for global npm, others not. 
  # install.sh uses 'run_cmd' which doesn't default to sudo unless needed, 
  # but 'npm install -g' usually needs sudo if prefix is /usr.
  # If installed via nvm, sudo is not needed.
  if [[ "$(npm config get prefix)" == "/usr"* ]]; then
    $SUDO npm update -g
  else
    npm update -g
  fi
fi

if command -v bun >/dev/null 2>&1; then
  echo "== Updating Bun =="
  bun upgrade
fi

if command -v fwupdmgr >/dev/null 2>&1; then
  echo "== Updating Firmware =="
  $SUDO fwupdmgr refresh --force || true
  $SUDO fwupdmgr get-updates || true
  $SUDO fwupdmgr update || true
fi

# Ghostty update logic
TARGET_BIN="$HOME/.local/bin"
BUILD_DIR="$HOME/ghostty-source"

if [[ -d "$BUILD_DIR" ]]; then
    echo "== Updating Ghostty (Source) =="
    
    # Check for zig
    if ! command -v zig >/dev/null 2>&1; then
        echo "Zig not found, checking /usr/local/bin/zig or /opt..."
        # Attempt to find zig if not in path (install.sh logic)
        if [[ -x "/usr/local/bin/zig" ]]; then
            export PATH="/usr/local/bin:$PATH"
        fi
    fi
    
    if command -v zig >/dev/null 2>&1; then
        echo "Updating source in $BUILD_DIR..."
        (cd "$BUILD_DIR" && git pull)
        
        echo "Rebuilding Ghostty..."
        if (cd "$BUILD_DIR" && zig build -Doptimize=ReleaseFast); then
            echo "Installing Ghostty binary..."
            mkdir -p "$TARGET_BIN"
            install -m 755 "$BUILD_DIR/zig-out/bin/ghostty" "$TARGET_BIN/ghostty"
            
            if [[ -d "$BUILD_DIR/zig-out/share" ]]; then
                 echo "Installing Ghostty resources..."
                 mkdir -p "$HOME/.local/share"
                 cp -r "$BUILD_DIR/zig-out/share/"* "$HOME/.local/share/"
                 update-desktop-database "$HOME/.local/share/applications" || true
            fi
            echo "Ghostty updated successfully."
        else
            echo "Ghostty build failed."
        fi
    else
        echo "Zig not found. Cannot rebuild Ghostty."
    fi
fi

echo "== Update Complete =="

