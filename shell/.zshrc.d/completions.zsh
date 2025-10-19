# =============================================================================
# Completions
# =============================================================================

# Docker CLI completions
fpath=($HOME/.docker/completions $fpath)
autoload -Uz compinit
compinit

# Crush completions (dynamic)
if (( $+commands[crush] )); then
  source <(crush completion zsh)
fi

# Cliffy completions (dynamic)
if (( $+commands[cliffy] )); then
  source <(cliffy completion zsh)
fi

# BEX completions (if available)
[[ -f $HOME/.bex-completion.zsh ]] && source $HOME/.bex-completion.zsh
