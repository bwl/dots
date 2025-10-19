# =============================================================================
# Zsh Configuration Router
# =============================================================================
# Pure entry point that detects environment and delegates to appropriate config
#
# Environment-specific configs:
#   ~/.zshrc-tuios  - TUIOS terminal multiplexer (minimal, fast)
#   ~/.zshrc-claude - Claude Code CLI (ultra-minimal, no prompt)
#   ~/.zshrc-legacy - Default full-featured config (Zim + Powerlevel10k)
#
# Detection order:
#   1. TUIOS_WINDOW_ID → tuios
#   2. CLAUDE_CODE_SESSION → claude
#   3. Default → legacy

# Get the directory where this .zshrc file actually lives
# This resolves symlinks so we can source configs from the dotfiles directory
ZSHRC_DIR="${${(%):-%x}:A:h}"

# Detect environment and route to appropriate config
if [[ -n "$TUIOS_WINDOW_ID" ]]; then
  # TUIOS terminal multiplexer
  source "$ZSHRC_DIR/.zshrc-tuios"
  return
elif [[ -n "$CLAUDE_CODE_SESSION" ]]; then
  # Claude Code CLI terminal
  source "$ZSHRC_DIR/.zshrc-claude"
  return
else
  # Default: full-featured legacy config
  source "$ZSHRC_DIR/.zshrc-legacy"
  return
fi
