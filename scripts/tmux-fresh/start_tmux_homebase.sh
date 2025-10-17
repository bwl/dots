#!/usr/bin/env bash
set -euo pipefail

# Entrypoint script for starting a project-specific tmux session with AI coding assistants.
# Run from any project directory to create a session for that project.
#
# Usage:
#   cd /path/to/my-project
#   fresh              # Create session and attach (or attach if exists)
#   fresh explain      # Show tmux context for agents
#   fresh kill         # Kill the current tmux session
#
# Or install globally:
#   ln -s /Users/bwl/Developer/setup/scripts/start_tmux_homebase.sh /usr/local/bin/fresh

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is not installed or not discoverable in PATH." >&2
  echo "Install tmux (e.g. 'brew install tmux') and try again." >&2
  exit 1
fi

# Handle 'fresh explain' subcommand
if [[ "${1:-}" == "explain" ]]; then
  if [[ -z "${TMUX:-}" ]]; then
    echo "Not currently in a tmux session."
    echo ""
    echo "Run 'fresh' from a project directory to create a session, then attach:"
    echo "  cd /path/to/project"
    echo "  fresh"
    echo "  tmux attach -t <session-name>"
    exit 0
  fi

  # Get current tmux context
  CURRENT_SESSION=$(tmux display-message -p '#S')
  CURRENT_WINDOW=$(tmux display-message -p '#W')
  CURRENT_WINDOW_IDX=$(tmux display-message -p '#I')
  CURRENT_PANE=$(tmux display-message -p '#P')
  CURRENT_PATH=$(tmux display-message -p '#{pane_current_path}')
  CURRENT_CMD=$(tmux display-message -p '#{pane_current_command}')

  echo "=== TMUX Context ==="
  echo ""
  echo "Session:  $CURRENT_SESSION"
  echo "Window:   $CURRENT_WINDOW (index: $CURRENT_WINDOW_IDX)"
  echo "Pane:     $CURRENT_PANE"
  echo "Path:     $CURRENT_PATH"
  echo "Command:  $CURRENT_CMD"
  echo ""

  # Load agent profiles
  # Resolve symlink to get actual script location
  SCRIPT_PATH="${BASH_SOURCE[0]}"
  if [[ -L "$SCRIPT_PATH" ]]; then
    SCRIPT_PATH=$(readlink "$SCRIPT_PATH")
  fi
  SCRIPT_DIR=$(dirname "$SCRIPT_PATH")
  PROFILES_FILE="$SCRIPT_DIR/agent_profiles.json"

  if [[ -f "$PROFILES_FILE" ]] && command -v jq >/dev/null 2>&1; then
    echo "=== Agent Profiles ==="
    echo ""

    # Get list of windows and their names
    while IFS=: read -r win_idx win_name; do
      # Look up profile for this window
      profile=$(jq -r --arg name "$win_name" '.[$name] // empty' "$PROFILES_FILE" 2>/dev/null)

      if [[ -n "$profile" ]]; then
        agent_name=$(echo "$profile" | jq -r '.name // empty')
        model=$(echo "$profile" | jq -r '.model // empty')
        role=$(echo "$profile" | jq -r '.role // empty')
        description=$(echo "$profile" | jq -r '.description // empty')

        echo "  $win_name (window $win_idx)"
        [[ -n "$agent_name" ]] && echo "    Name:  $agent_name"
        [[ -n "$model" ]] && echo "    Model: $model"
        [[ -n "$role" ]] && echo "    Role:  $role"
        [[ -n "$description" ]] && echo "    Use:   $description"
        echo ""
      fi
    done < <(tmux list-windows -t "$CURRENT_SESSION" -F "#{window_index}:#{window_name}")
  fi

  echo "=== Available Windows ==="
  echo ""
  tmux list-windows -t "$CURRENT_SESSION" -F "  #{window_index}: #{window_name} - #{pane_current_command} [#{pane_current_path}]"
  echo ""
  echo "=== Interacting with Other Panes ==="
  echo ""
  echo "Send command to a window:"
  echo "  tmux send-keys -t \"$CURRENT_SESSION:<window>.0\" \"command\" C-m"
  echo ""
  echo "Read output from a window:"
  echo "  tmux capture-pane -t \"$CURRENT_SESSION:<window>.0\" -p -S -50"
  echo ""
  echo "Examples:"
  echo "  # Send to codex window (index 1)"
  echo "  tmux send-keys -t \"$CURRENT_SESSION:1.0\" \"explain the architecture\" C-m"
  echo ""
  echo "  # Read from tasks window (index 4)"
  echo "  tmux capture-pane -t \"$CURRENT_SESSION:4.0\" -p -S -20"
  echo ""
  echo "Full command reference: ~/Developer/setup/docs/tmux-agent-commands.md"
  exit 0
fi

# Handle 'fresh kill' subcommand
if [[ "${1:-}" == "kill" ]]; then
  if [[ -z "${TMUX:-}" ]]; then
    echo "Not currently in a tmux session." >&2
    exit 1
  fi

  # Get current session name
  CURRENT_SESSION=$(tmux display-message -p '#S')

  echo "Killing tmux session '$CURRENT_SESSION'..."

  # Kill the session (this will also detach us)
  tmux kill-session -t "$CURRENT_SESSION"

  exit 0
fi

# Capture the current working directory as the project directory
PROJECT_DIR=$(pwd)

# Generate session name from directory name (sanitize for tmux)
DIR_NAME=$(basename "$PROJECT_DIR")
SESSION_NAME=$(echo "$DIR_NAME" | sed 's/[^a-zA-Z0-9_-]/_/g')

# If the session already exists, attach to it
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
  if [[ -n "${TMUX:-}" ]]; then
    # Already in tmux, switch to the session
    tmux switch-client -t "$SESSION_NAME"
  else
    # Not in tmux, attach to the session
    exec tmux attach -t "$SESSION_NAME"
  fi
  exit 0
fi

# Helper function to create a tmux window and optionally run a command
create_window() {
  local window_name=$1
  local window_cmd=$2
  local window_idx=$3
  local add_tasks_sidebar=$4

  if (( window_idx == 0 )); then
    # Create the first window as part of the initial session, in the project directory
    tmux new-session -d -s "$SESSION_NAME" -n "$window_name" -c "$PROJECT_DIR"
  else
    # Create subsequent windows in the project directory
    tmux new-window -t "$SESSION_NAME" -n "$window_name" -c "$PROJECT_DIR"
  fi

  # Send the command if provided
  if [[ -n "$window_cmd" ]]; then
    # Source shell rc file to load aliases and functions
    # This ensures commands defined as aliases (not in PATH) work correctly
    local shell_init=""
    if [[ "$SHELL" == */zsh ]] && [[ -f "$HOME/.zshrc" ]]; then
      shell_init="source ~/.zshrc && "
    elif [[ "$SHELL" == */bash ]] && [[ -f "$HOME/.bashrc" ]]; then
      shell_init="source ~/.bashrc && "
    fi

    tmux send-keys -t "${SESSION_NAME}:${window_idx}.0" "${shell_init}${window_cmd}" C-m
  fi

  # Add tasks sidebar if requested
  if [[ "$add_tasks_sidebar" == "true" ]]; then
    # Split window vertically: left pane (70%) for agent, right pane (30%) for tasks
    tmux split-window -t "${SESSION_NAME}:${window_idx}.0" -h -p 30 -c "$PROJECT_DIR"

    # In the right pane (pane 1), run a loop to auto-refresh tasks
    # Use unbuffer to preserve ANSI colors/formatting when capturing output
    # Capture output first, then clear and display to avoid flicker
    local shell_init=""
    if [[ "$SHELL" == */zsh ]] && [[ -f "$HOME/.zshrc" ]]; then
      shell_init="source ~/.zshrc && "
    elif [[ "$SHELL" == */bash ]] && [[ -f "$HOME/.bashrc" ]]; then
      shell_init="source ~/.bashrc && "
    fi

    # Build tb command with storage-dir flag if .taskbook exists (matches tb function logic)
    local tb_cmd="tb"
    if [[ -d "$PROJECT_DIR/.taskbook" ]]; then
      tb_cmd="tb --storage-dir ./.taskbook"
    fi

    tmux send-keys -t "${SESSION_NAME}:${window_idx}.1" "${shell_init}while true; do output=\$(unbuffer $tb_cmd); clear; echo \"\$output\"; sleep 2; done" C-m

    # Set focus back to the left pane (pane 0)
    tmux select-pane -t "${SESSION_NAME}:${window_idx}.0"
  fi
}

# Window configuration: name, command, environment variable override
window_names=(claude codex cliffy git tasks)
window_cmds=(
  "${TMUX_CLAUDE_CMD:-claude}"
  "${TMUX_CODEX_CMD:-codex}"
  "${TMUX_CLIFFY_CMD:-cliffy}"
  "${TMUX_GIT_CMD:-lazygit}"
  "${TMUX_TASK_CMD:-tb}"
)

# Create all windows
for idx in "${!window_names[@]}"; do
  # Add tasks sidebar to all windows except the tasks window itself
  if [[ "${window_names[$idx]}" == "tasks" ]]; then
    create_window "${window_names[$idx]}" "${window_cmds[$idx]}" "$idx" "false"
  else
    create_window "${window_names[$idx]}" "${window_cmds[$idx]}" "$idx" "true"
  fi
done

tmux display-message -t "$SESSION_NAME" "Session '$SESSION_NAME' ready in $PROJECT_DIR"

# Auto-attach to the newly created session
if [[ -n "${TMUX:-}" ]]; then
  # Already in tmux, switch to the new session
  tmux switch-client -t "$SESSION_NAME"
else
  # Not in tmux, attach to the new session
  exec tmux attach -t "$SESSION_NAME"
fi
