# =============================================================================
# External Tool Integrations
# =============================================================================

# Pyenv initialization
if command -v pyenv &> /dev/null; then
  eval "$(pyenv init -)"
  eval "$(pyenv virtualenv-init -)"
fi

# Broot launcher
[[ -f $HOME/.config/broot/launcher/bash/br ]] && source $HOME/.config/broot/launcher/bash/br

# Langflow environment
[[ -f $HOME/.langflow/uv/env ]] && source $HOME/.langflow/uv/env

# Zoxide (smart cd)
if command -v zoxide &> /dev/null; then
  eval "$(zoxide init zsh)"
fi

# Tmuxifier
if command -v tmuxifier &> /dev/null; then
  eval "$(tmuxifier init -)"
fi
