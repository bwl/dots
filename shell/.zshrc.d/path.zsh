# =============================================================================
# PATH Configuration
# =============================================================================
# Note: .zshenv handles basic PATH for non-interactive shells
# This file ensures interactive shells have all paths

export PATH="/opt/homebrew/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export PATH="$HOME/Developer/cliffy/bin:$PATH"
export PATH="/opt/homebrew/opt/openjdk/bin:$PATH"

# Go
export GOPATH="${GOPATH:-$HOME/go}"
export PATH="$GOPATH/bin:$PATH"
export PATH="/usr/local/go/bin:$PATH"

# Pyenv
export PYENV_ROOT="$HOME/.pyenv"
[[ -d "$PYENV_ROOT/bin" ]] && export PATH="$PYENV_ROOT/bin:$PATH"

# Tmuxifier
export TMUXIFIER_PATH="$HOME/.tmuxifier"
[[ -d "$TMUXIFIER_PATH/bin" ]] && export PATH="$TMUXIFIER_PATH/bin:$PATH"
