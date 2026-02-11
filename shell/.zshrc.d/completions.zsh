# =============================================================================
# Completions
# =============================================================================

# Completion directories
fpath=($HOME/.zfunc $HOME/.docker/completions $fpath)

# Generate dynamic completions only if missing (rm ~/.zfunc/_* to refresh)
(( $+commands[ncli] )) && [[ ! -f $HOME/.zfunc/_ncli ]] && ncli completions zsh > $HOME/.zfunc/_ncli 2>/dev/null
(( $+commands[icli] )) && [[ ! -f $HOME/.zfunc/_icli ]] && icli completions zsh > $HOME/.zfunc/_icli 2>/dev/null
(( $+commands[crush] )) && [[ ! -f $HOME/.zfunc/_crush ]] && crush completion zsh > $HOME/.zfunc/_crush 2>/dev/null
(( $+commands[cliffy] )) && [[ ! -f $HOME/.zfunc/_cliffy ]] && cliffy completion zsh > $HOME/.zfunc/_cliffy 2>/dev/null
(( $+commands[karl] )) && [[ ! -f $HOME/.zfunc/_karl ]] && karl completions zsh > $HOME/.zfunc/_karl 2>/dev/null

source "/opt/homebrew/opt/fzf-tab/share/fzf-tab/fzf-tab.zsh"

autoload -Uz compinit
compinit -C  # Use cached completions if available
