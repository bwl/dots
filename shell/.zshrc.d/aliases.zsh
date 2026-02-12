# =============================================================================
# Aliases
# =============================================================================

# --- Modern CLI replacements ---
alias ls="eza --icons=always"
alias ll="eza -la --icons=always --git"
alias lt="eza -la --icons=always --git --tree --level=2"
alias cat="bat"
alias top="btop"
alias du="dust"
alias df="duf"
alias ps="procs"

# --- Navigation ---
alias ..="cd .."
alias ...="cd ../.."

# --- Git ---
alias g="git"
alias gd="git diff"
alias gl="git log --oneline -20"
alias lg="lazygit"

# --- Tools ---
alias v="nvim"
alias vi="nvim"
alias vim="nvim"
alias y="yazi"
alias sg="ast-grep"
alias reloadcli="exec zsh"

# --- Project-specific ---
alias tarot="$HOME/Developer/tarot/tarot"
