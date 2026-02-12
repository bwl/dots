# =============================================================================
# .zprofile - Login Shell Setup
# =============================================================================

# Homebrew shell environment (macOS only)
if command -v brew &>/dev/null; then
  eval "$(brew shellenv)"
fi
