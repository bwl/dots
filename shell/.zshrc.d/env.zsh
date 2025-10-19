# =============================================================================
# Environment Variables
# =============================================================================

# Homebrew
export HOMEBREW_CASK_OPTS="--appdir=$HOME/Applications"
export HOMEBREW_NO_AUTO_UPDATE=TRUE

# Go
export GOPATH=$HOME/go

# FZF - Use ripgrep for file searching
export FZF_DEFAULT_COMMAND='rg --files --hidden --glob "!.git"'
