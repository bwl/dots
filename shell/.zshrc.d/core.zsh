# =============================================================================
# Core Zsh Configuration
# =============================================================================
# Essential shell behavior: history, keybindings, Zim framework
#
# Note: The Zim `environment` module sets these options (not duplicated here):
#   SHARE_HISTORY, HIST_IGNORE_DUPS, HIST_IGNORE_SPACE, HIST_VERIFY,
#   AUTO_CD, AUTO_PUSHD, EXTENDED_GLOB, NO_CLOBBER, INTERACTIVE_COMMENTS
#
# Note: The Zim `input` module handles these (not duplicated here):
#   Key bindings (Home/End/Delete/PageUp/PageDown/Ctrl-arrows/Backspace)
#   History-substring-search Up/Down/Ctrl-P/Ctrl-N bindings (deferred)

# =============================================================================
# History Configuration (additions beyond environment module)
# =============================================================================

setopt EXTENDED_HISTORY       # Write the history file in the ':start:elapsed;command' format
setopt INC_APPEND_HISTORY     # Write to the history file immediately, not when the shell exits
setopt HIST_IGNORE_ALL_DUPS   # Delete an old recorded event if a new event is a duplicate
setopt HIST_SAVE_NO_DUPS      # Do not write a duplicate event to the history file
setopt APPEND_HISTORY         # Append to history file (default)
setopt HIST_NO_STORE          # Don't store history commands
setopt HIST_REDUCE_BLANKS     # Remove superfluous blanks from each command line

HISTFILE="$HOME/.zsh_history"
HISTSIZE=10000000
SAVEHIST=10000000
HIST_STAMPS="yyyy-mm-dd"
HISTORY_IGNORE="(ls|cd|pwd|exit)*"

# =============================================================================
# Input/Output Configuration
# =============================================================================

# Set editor default keymap to emacs (`-e`) or vi (`-v`)
bindkey -e

# Remove path separator from WORDCHARS
WORDCHARS=${WORDCHARS//[\/]}

# =============================================================================
# Zim Framework Configuration
# =============================================================================

# zsh-autosuggestions
ZSH_AUTOSUGGEST_MANUAL_REBIND=1

# zsh-syntax-highlighting
ZSH_HIGHLIGHT_HIGHLIGHTERS=(main brackets)

# termtitle
zstyle ':zim:termtitle' hooks 'precmd' 'preexec'
zstyle ':zim:termtitle' format '%~'
zstyle ':zim:termtitle' format-preexec '${1}'

# =============================================================================
# Zim Framework Initialization
# =============================================================================

ZIM_HOME=${ZDOTDIR:-${HOME}}/.zim

# Download zimfw plugin manager if missing
if [[ ! -e ${ZIM_HOME}/zimfw.zsh ]]; then
  if (( ${+commands[curl]} )); then
    curl -fsSL --create-dirs -o "${ZIM_HOME}/zimfw.zsh" \
        https://github.com/zimfw/zimfw/releases/latest/download/zimfw.zsh
  else
    mkdir -p "${ZIM_HOME}" && wget -nv -O "${ZIM_HOME}/zimfw.zsh" \
        https://github.com/zimfw/zimfw/releases/latest/download/zimfw.zsh
  fi
fi

# Install missing modules, and update ${ZIM_HOME}/init.zsh if missing or outdated
if [[ ! ${ZIM_HOME}/init.zsh -nt ${ZDOTDIR:-${HOME}}/.zimrc ]]; then
  source "${ZIM_HOME}/zimfw.zsh" init -q
fi

# Initialize modules
source "${ZIM_HOME}/init.zsh"
