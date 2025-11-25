# =============================================================================
# External Tool Integrations
# =============================================================================

# Pyenv initialization (--no-rehash for faster startup)
if command -v pyenv &> /dev/null; then
  eval "$(pyenv init - --no-rehash)"
  eval "$(pyenv virtualenv-init -)"
fi

# Broot launcher
[[ -f "$HOME/.config/broot/launcher/bash/br" ]] && source "$HOME/.config/broot/launcher/bash/br"

# Langflow environment
[[ -f "$HOME/.langflow/uv/env" ]] && source "$HOME/.langflow/uv/env"

# Zoxide (smart cd)
if command -v zoxide &> /dev/null; then
  eval "$(zoxide init zsh)"
fi

# Tmuxifier
if command -v tmuxifier &> /dev/null; then
  eval "$(tmuxifier init -)"
fi

# Try - Experiment directory manager
if [[ -x "$HOME/.local/try.rb" ]]; then
  eval "$(ruby "$HOME/.local/try.rb" init ~/src/tries)"
elif command -v try &> /dev/null; then
  eval "$(try init ~/src/tries)"
fi
