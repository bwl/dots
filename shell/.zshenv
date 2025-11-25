# =============================================================================
# .zshenv - Environment Setup (Always Loaded)
# =============================================================================
# This file is sourced for ALL shells (login, interactive, non-interactive, scripts)
# Keep this minimal and fast - heavy lifting is in .zshrc.d/

# =============================================================================
# Essential PATH (Non-Interactive Shells)
# =============================================================================
# Scripts and non-interactive shells need basic PATH to find tools

export PATH="/opt/homebrew/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/bin:$PATH"

# =============================================================================
# Secrets Management
# =============================================================================

# Source local secrets file if it exists (not tracked in git)
[[ -f "$HOME/dotfiles/shell/.zshenv-secrets" ]] && source "$HOME/dotfiles/shell/.zshenv-secrets"
