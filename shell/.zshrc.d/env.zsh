# =============================================================================
# Environment Variables
# =============================================================================

# Editor
export EDITOR="nvim"

# Homebrew (macOS only)
if [[ "$(uname)" == "Darwin" ]]; then
  export HOMEBREW_CASK_OPTS="--appdir=$HOME/Applications"
  export HOMEBREW_NO_AUTO_UPDATE=TRUE
fi

# Go
export GOPATH=$HOME/go

# FZF - Use ripgrep for file searching
export FZF_DEFAULT_COMMAND='rg --files --hidden --glob "!.git"'
