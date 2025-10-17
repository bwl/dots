# =============================================================================
# .zshenv - Environment Setup (Always Loaded)
# =============================================================================
# This file is sourced for ALL shells (login, interactive, non-interactive, scripts)
# Put PATH and environment variables here

# =============================================================================
# PATH Configuration
# =============================================================================

export PATH="/opt/homebrew/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export PATH="$HOME/Developer/cliffy/bin:$PATH"

# Pyenv
export PYENV_ROOT="$HOME/.pyenv"
[[ -d $PYENV_ROOT/bin ]] && export PATH="$PYENV_ROOT/bin:$PATH"

# Go
export GOPATH=$HOME/go
export PATH="$GOPATH/bin:$PATH"
export PATH="/usr/local/go/bin:$PATH"

# Tmuxifier
export TMUXIFIER_PATH="$HOME/.tmuxifier"
[[ -d $TMUXIFIER_PATH/bin ]] && export PATH="$TMUXIFIER_PATH/bin:$PATH"

# =============================================================================
# Environment Variables
# =============================================================================

# Homebrew
export HOMEBREW_CASK_OPTS="--appdir=$HOME/Applications"
export HOMEBREW_NO_AUTO_UPDATE=TRUE

# FZF - Use ripgrep for file searching
export FZF_DEFAULT_COMMAND='rg --files --hidden --glob "!.git"'

# =============================================================================
# Secrets from Bitwarden
# =============================================================================

# Disabled for now - uncomment when needed
# Bitwarden session management with Touch ID via macOS Keychain
# if command -v bw &> /dev/null; then
#   # Check if vault is already unlocked
#   if ! bw unlock --check &>/dev/null; then
#     # Retrieve password from macOS Keychain (will prompt for Touch ID)
#     BW_PASSWORD=$(security find-generic-password -a "$USER" -s bw_master_password -w 2>/dev/null)
#
#     if [[ -n "$BW_PASSWORD" ]]; then
#       # Unlock Bitwarden with password from keychain
#       export BW_SESSION="$(echo "$BW_PASSWORD" | bw unlock --raw 2>/dev/null)"
#       unset BW_PASSWORD
#     else
#       # Keychain entry not found, fall back to manual password
#       echo "⚠️  Bitwarden password not found in keychain. Enter master password:"
#       export BW_SESSION="$(bw unlock --raw)"
#     fi
#   fi
#
#   # Only retrieve secrets if vault is unlocked
#   if bw unlock --check &>/dev/null; then
#     export CLIFFY_OPENROUTER_API_KEY="$(bw get password 'Cliffy OpenRouter API Key')"
#   fi
# fi
