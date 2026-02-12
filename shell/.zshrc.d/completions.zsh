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
(( $+commands[bun] )) && [[ ! -f $HOME/.zfunc/_bun ]] && bun completions > $HOME/.zfunc/_bun 2>/dev/null
(( $+commands[ast-grep] )) && [[ ! -f $HOME/.zfunc/_ast-grep ]] && ast-grep completions zsh > $HOME/.zfunc/_ast-grep 2>/dev/null

# fzf-tab (platform-aware paths)
if [[ -f "/opt/homebrew/opt/fzf-tab/share/fzf-tab/fzf-tab.zsh" ]]; then
  source "/opt/homebrew/opt/fzf-tab/share/fzf-tab/fzf-tab.zsh"
elif [[ -f "/usr/share/zsh/plugins/fzf-tab/fzf-tab.zsh" ]]; then
  source "/usr/share/zsh/plugins/fzf-tab/fzf-tab.zsh"
fi

# compinit is handled by Zim's completion module (in core.zsh)
# after all fpath entries are registered
