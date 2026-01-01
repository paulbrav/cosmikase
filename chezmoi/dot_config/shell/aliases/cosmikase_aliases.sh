# Cosmikase shell aliases

# Navigation
alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'

# Git shortcuts (additional to git config aliases)
alias g='git'
alias gs='git status'
alias ga='git add'
alias gc='git commit'
alias gp='git push'
alias gl='git pull'
alias gd='git diff'
alias gco='git checkout'
alias gb='git branch'
alias glog='git log --oneline --graph --decorate'

# Safety
alias rm='rm -i'
alias cp='cp -i'
alias mv='mv -i'

# Utility
alias h='history'
alias j='jobs -l'
alias path='echo -e ${PATH//:/\\n}'

# Quick edit configs
alias zshrc='${EDITOR:-nvim} ~/.zshrc'
alias bashrc='${EDITOR:-nvim} ~/.bashrc'

# Short tool aliases
alias n='nvim'
alias d='docker'
alias f='fdfind'
alias r='rg'

# Fuzzy finder with preview
alias ff='fzf --preview "bat --color=always --style=numbers --line-range=:500 {}"'

# TUI applications
alias lzg='lazygit'
alias lzd='lazydocker'

# --- Omakub-style Utility Functions ---

# Archive utilities
compress() {
    if [[ -z "$1" ]]; then
        echo "Usage: compress <file-or-directory>" >&2
        return 1
    fi
    # Use tar to create a compressed archive
    tar -czf "${1%/}.tar.gz" "$1" && echo "Created ${1%/}.tar.gz"
}

decompress() {
    if [[ -z "$1" ]]; then
        echo "Usage: decompress <archive.tar.gz>" >&2
        return 1
    fi
    if [[ ! -f "$1" ]]; then
        echo "Error: File '$1' not found." >&2
        return 1
    fi
    tar -xzf "$1"
}

# Video conversion (requires ffmpeg)
webm2mp4() {
    if [[ -z "$1" ]] || [[ ! -f "$1" ]]; then
        echo "Usage: webm2mp4 <input.webm>" >&2
        return 1
    fi
    if ! command -v ffmpeg >/dev/null 2>&1; then
        echo "Error: ffmpeg is not installed." >&2
        return 1
    fi
    ffmpeg -i "$1" -c:v libx264 -crf 23 -c:a aac "${1%.webm}.mp4"
}

# Write ISO to SD card
iso2sd() {
    if [[ $# -ne 2 ]]; then
        echo "Usage: iso2sd <input.iso> </dev/sdX>" >&2
        echo "WARNING: This will overwrite the target device!" >&2
        return 1
    fi
    if [[ ! -f "$1" ]]; then
        echo "Error: ISO file '$1' not found." >&2
        return 1
    fi
    echo "This will destroy all data on $2. Are you sure? (y/N)"
    read -r confirm
    if [[ "$confirm" != "y" ]] && [[ "$confirm" != "Y" ]]; then
        echo "Aborted."
        return 1
    fi
    sudo dd if="$1" of="$2" bs=4M status=progress oflag=sync
}

# Docker utilities
alias dps='docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"'
dlog() {
    if [[ -z "$1" ]]; then
        echo "Usage: dlog <container-name>" >&2
        return 1
    fi
    docker logs -f "$1";
}
dexec() {
    if [[ -z "$1" ]]; then
        echo "Usage: dexec <container-name> [command]" >&2
        return 1
    fi
    docker exec -it "$1" "${2:-bash}";
}

# Web app launcher management
# Create a desktop launcher for a web application
web2app() {
    if [[ $# -lt 2 ]]; then
        echo "Usage: web2app <Name> <URL> [Icon URL]" >&2
        echo "Example: web2app 'My App' 'https://example.com' 'https://example.com/icon.png'" >&2
        return 1
    fi

    local name="$1"
    local url="$2"
    local icon_url="${3:-}"
    local safe_name="${name// /-}"
    local desktop_file="$HOME/.local/share/applications/${safe_name}.desktop"
    local icon_path=""

    # Download icon if URL provided
    if [[ -n "$icon_url" ]]; then
        local icon_dir="$HOME/.local/share/icons/web2app"
        mkdir -p "$icon_dir"
        icon_path="$icon_dir/${safe_name}.png"
        if ! curl -sL "$icon_url" -o "$icon_path"; then
            echo "Warning: Failed to download icon, using default" >&2
            icon_path="web-browser"
        fi
    else
        icon_path="web-browser"
    fi

    # Create desktop file
    cat > "$desktop_file" << EOF
[Desktop Entry]
Name=${name}
Exec=xdg-open ${url}
Icon=${icon_path}
Type=Application
Categories=Network;WebBrowser;
StartupNotify=true
EOF

    chmod +x "$desktop_file"
    echo "Created web app launcher: $desktop_file"
}

# Remove a web app launcher
web2app-remove() {
    if [[ -z "$1" ]]; then
        echo "Usage: web2app-remove <Name>" >&2
        return 1
    fi

    local name="$1"
    local safe_name="${name// /-}"
    local desktop_file="$HOME/.local/share/applications/${safe_name}.desktop"
    local icon_path="$HOME/.local/share/icons/web2app/${safe_name}.png"

    if [[ -f "$desktop_file" ]]; then
        rm "$desktop_file"
        echo "Removed: $desktop_file"
    else
        echo "Desktop file not found: $desktop_file" >&2
        return 1
    fi

    if [[ -f "$icon_path" ]]; then
        rm "$icon_path"
        echo "Removed icon: $icon_path"
    fi
}

