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





