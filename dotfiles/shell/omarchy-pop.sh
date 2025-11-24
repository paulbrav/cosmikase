# omarchy-pop shell helpers
if command -v fdfind >/dev/null 2>&1 && ! command -v fd >/dev/null 2>&1; then
  alias fd=fdfind
fi

if command -v zoxide >/dev/null 2>&1; then
  eval "$(zoxide init bash 2>/dev/null || zoxide init zsh 2>/dev/null)"
fi

if command -v starship >/dev/null 2>&1; then
  eval "$(starship init bash 2>/dev/null || starship init zsh 2>/dev/null)"
fi

if command -v eza >/dev/null 2>&1; then
  alias ls='eza --group-directories-first --icons=auto'
  alias ll='eza -lah --group-directories-first --icons=auto'
fi

# Alias for Flatpak Obsidian
if flatpak list 2>/dev/null | grep -q "md.obsidian.Obsidian"; then
  alias obsidian='flatpak run md.obsidian.Obsidian'
fi
