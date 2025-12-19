# =============================================================================
# External Tool Integrations (lazy-loaded for fast startup)
# =============================================================================

# --- Pyenv (lazy) ---
if (( $+commands[pyenv] )); then
  _lazy_pyenv() {
    unfunction pyenv python python3 pip pip3 2>/dev/null
    eval "$(command pyenv init - --no-rehash)"
    eval "$(command pyenv virtualenv-init -)"
  }
  pyenv() { _lazy_pyenv; command pyenv "$@" }
  python() { _lazy_pyenv; command python "$@" }
  python3() { _lazy_pyenv; command python3 "$@" }
  pip() { _lazy_pyenv; command pip "$@" }
  pip3() { _lazy_pyenv; command pip3 "$@" }
fi

# --- Zoxide (lazy) ---
if (( $+commands[zoxide] )); then
  z() {
    unfunction z zi 2>/dev/null
    eval "$(zoxide init zsh)"
    z "$@"
  }
  zi() {
    unfunction z zi 2>/dev/null
    eval "$(zoxide init zsh)"
    zi "$@"
  }
fi

# --- Tmuxifier (lazy) ---
if (( $+commands[tmuxifier] )); then
  tmuxifier() {
    unfunction tmuxifier 2>/dev/null
    eval "$(command tmuxifier init -)"
    tmuxifier "$@"
  }
fi

# --- Try (lazy) ---
if [[ -x "$HOME/.local/try.rb" ]]; then
  try() {
    unfunction try 2>/dev/null
    eval "$(ruby "$HOME/.local/try.rb" init ~/src/tries)"
    try "$@"
  }
elif (( $+commands[try] )); then
  try() {
    unfunction try 2>/dev/null
    eval "$(command try init ~/src/tries)"
    try "$@"
  }
fi

# --- Eager loads (fast, no subshells) ---
# Broot launcher
[[ -f "$HOME/.config/broot/launcher/bash/br" ]] && source "$HOME/.config/broot/launcher/bash/br"

# Langflow environment
[[ -f "$HOME/.langflow/uv/env" ]] && source "$HOME/.langflow/uv/env"
