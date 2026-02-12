# =============================================================================
# Zsh Configuration - Modular Loader
# =============================================================================
# Sources configuration from .zshrc.d/*.zsh in sorted order
#
# Module loading order (alphabetical):
#   aliases.zsh      - Aliases
#   completions.zsh  - Command completions
#   core.zsh         - History, keybindings, Zim framework
#   env.zsh          - Environment variables
#   functions.zsh    - Shell functions
#   integrations.zsh - External tool integrations (pyenv, zoxide, etc.)
#   path.zsh         - PATH configuration
#
# For Claude Code sessions (CLAUDECODE or CLAUDE_CODE_SESSION env var),
# a minimal config is loaded instead (no prompt, faster startup).

# Get the directory where this .zshrc file actually lives
# This resolves symlinks so we can source configs from the dots directory
ZSHRC_DIR="${${(%):-%x}:A:h}"

# =============================================================================
# Claude Code Detection (Minimal Mode)
# =============================================================================
# Claude Code sessions need minimal config - no prompt customization

if [[ -n "$CLAUDECODE" ]] || [[ -n "$CLAUDE_CODE_SESSION" ]]; then
  # Glob handling - prevent errors on unmatched patterns
  setopt NO_NOMATCH    # Don't error when glob has no matches (pass through as literal)
  setopt NULL_GLOB     # Failed globs expand to nothing instead of error

  # Minimal history
  setopt EXTENDED_HISTORY INC_APPEND_HISTORY SHARE_HISTORY HIST_IGNORE_DUPS HIST_IGNORE_SPACE
  HISTFILE="$HOME/.zsh_history"
  HISTSIZE=10000000
  SAVEHIST=10000000

  # Essential PATH
  [[ "$(uname)" == "Darwin" ]] && export PATH="/opt/homebrew/bin:$PATH"
  export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$HOME/bin:$HOME/.bun/bin:$PATH"
  export GOPATH="${GOPATH:-$HOME/go}"
  export PATH="$GOPATH/bin:$PATH"

  # Essential functions
  tb() { [[ -d .taskbook ]] && command tb --storage-dir ./.taskbook "$@" || command tb "$@"; }
  git() { (( $+commands[attempt] )) && command attempt git "$@" || command git "$@"; }

  # Minimal integrations
  command -v pyenv &>/dev/null && eval "$(pyenv init - --no-rehash)"
  command -v zoxide &>/dev/null && eval "$(zoxide init zsh)"

  return
fi

# =============================================================================
# Powerlevel10k Instant Prompt
# =============================================================================
# Must be near top of file, before any output

if [[ -r "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh" ]]; then
  source "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh"
fi

# =============================================================================
# Load Modular Configuration
# =============================================================================

for config in "$ZSHRC_DIR/.zshrc.d"/*.zsh(N); do
  source "$config"
done

# =============================================================================
# Powerlevel10k Theme
# =============================================================================

# =============================================================================
# Machine-Local Overrides
# =============================================================================

[[ -f ~/.zshrc.local ]] && source ~/.zshrc.local

# =============================================================================
# Powerlevel10k Theme
# =============================================================================

[[ -f "$ZSHRC_DIR/.p10k.zsh" ]] && source "$ZSHRC_DIR/.p10k.zsh"

# To customize prompt, run `p10k configure` or edit ~/dots/shell/.p10k.zsh.
[[ ! -f ~/dots/shell/.p10k.zsh ]] || source ~/dots/shell/.p10k.zsh
