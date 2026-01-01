# Omarchy-pop shell aliases

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





