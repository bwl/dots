#!/opt/homebrew/bin/bash
set -euo pipefail

# Leaf: AI-optimized tmux workspace with dedicated execution windows
#
# Creates a tmux session with:
# - `coord` window running Claude
# - `logs`, `otel`, `codex`, `cliffy` windows for observability and task lanes
# - Optional additional windows defined via name:command pairs
#
# Usage:
#   leaf init                                       # Create session with default windows
#   leaf init build:'cargo watch' log:'watch log'   # Add project-specific windows
#   leaf explain                                    # Show context and window info
#   leaf kill                                       # Kill current leaf session
#   leaf list                                       # List all leaf sessions
#   leaf attach <session>                           # Attach to specific leaf session

VERSION="0.1"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_error() {
  echo -e "${RED}Error:${NC} $1" >&2
}

log_info() {
  echo -e "${BLUE}$1${NC}"
}

log_success() {
  echo -e "${GREEN}$1${NC}"
}

log_warn() {
  echo -e "${YELLOW}$1${NC}"
}

# Check if tmux is installed
check_tmux() {
  if ! command -v tmux >/dev/null 2>&1; then
    log_error "tmux is not installed or not discoverable in PATH."
    echo "Install tmux (e.g. 'brew install tmux') and try again." >&2
    exit 1
  fi
}

# Show usage information
usage() {
  cat <<EOF
Leaf v${VERSION} - AI-optimized tmux workspace

Usage:
  leaf init [panel:command ...]       Create/attach session with optional named windows
  leaf plan [--resume id]             Start or attach to Codex REPL
  leaf plan attach [--resume id]      Alias for Codex REPL attach
  leaf plan send "<message>"          Send prompt to Codex REPL (REPL auto-starts)
  leaf do codex <task> [options]      Run Codex YAML task (one-off exec)
  leaf do <subcommand> [options]      Execute various actions (tasks, commits, etc.)
  leaf explain                        Show context and window information
  leaf kill                           Kill current leaf session
  leaf list                           List all leaf sessions
  leaf attach <session>               Attach to specific leaf session
  leaf status clean                   Remove persistent status windows
  leaf help                           Show this help message
  leaf version                        Show version

Examples:
  # Create session with default windows (coord/logs/otel/codex/cliffy)
  leaf init

  # Create session with extra execution windows
  leaf init build:'cargo watch -x build' test:'npm test' serve:'bun run dev'

  # Launch Codex REPL (will restart if not running)
  leaf plan
  leaf plan --resume 20241019T1330

  # Send prompt to Codex REPL (quotes preserve spaces; use $'...' for multi-line)
  leaf plan send "Summarize current blockers"
  leaf plan send $'List TODOs:\n- docs\n- tests'

  # Run a Codex YAML task (uses .leaf/plans/<task>.yml)
  leaf do codex refactor-plan-to-repl

  # Execute batch tasks via cliffy
  leaf do tasks "analyze auth.go" "review db.go" "check tests"
  leaf do tasks --tasks-file .leaf/tasks.txt --max-concurrent 3

  # Show current leaf context (useful for AI agents)
  leaf explain

  # List all leaf sessions
  leaf list

  # Kill current session
  leaf kill

Window Spec Syntax:
  Windows are specified as name:command pairs separated by spaces (CLI flag remains panel:command).
  - Name: alphanumeric, dashes, underscores (e.g., build, test-unit, serve_dev)
  - Command: any shell command (use quotes if it contains spaces)

  Examples:
    build:'cargo watch'
    test:'npm run test:watch'
    serve:'bun --hot run dev'
    log:'tail -f /var/log/app.log'

Subcommands for 'leaf do':
  codex       Run Codex plan YAML (one-off exec)
  tasks       Execute batch tasks via cliffy (parallel execution)
  (future: commit, release, etc.)

EOF
}

# Convert string into slug (lowercase, dash separated)
slugify() {
  local input="$1"
  local slug
  slug=$(echo "$input" | tr '[:upper:]' '[:lower:]')
  slug=$(echo "$slug" | tr ' ' '-')
  slug=$(echo "$slug" | sed 's/[^a-z0-9_-]/-/g')
  slug=$(echo "$slug" | sed 's/-\+/-/g; s/^-//; s/-$//')
  echo "$slug"
}

resolve_project_dir() {
  local session="${1:-}"
  local pane="${2:-}"
  local project_path=""

  if [[ -n "$session" ]]; then
    local env_line
    env_line=$(tmux show-environment -t "$session" LEAF_PROJECT_ROOT 2>/dev/null || true)
    if [[ -n "$env_line" ]]; then
      project_path=$(echo "$env_line" | cut -d= -f2-)
    fi
  fi

  if [[ -z "$project_path" && -n "$session" ]]; then
    project_path=$(tmux display-message -t "$session" -p '#{session_path}' 2>/dev/null || true)
  fi

  if [[ -z "$project_path" && -n "$pane" ]]; then
    project_path=$(tmux display-message -t "$pane" -p '#{pane_current_path}' 2>/dev/null || true)
  fi

  if [[ -z "$project_path" && -n "${TMUX_PANE:-}" ]]; then
    project_path=$(tmux display-message -t "$TMUX_PANE" -p '#{pane_current_path}' 2>/dev/null || true)
  fi

  if [[ -z "$project_path" && -n "${TMUX:-}" ]]; then
    project_path=$(tmux display-message -p '#{pane_current_path}' 2>/dev/null || true)
  fi

  if [[ -z "$project_path" && -z "$session" ]]; then
    project_path=$(pwd 2>/dev/null || true)
  fi

  if [[ -z "$project_path" ]]; then
    return 1
  fi

  printf '%s\n' "$project_path"
}

# Find window index by name, returns empty string if not found
find_window_index_by_name() {
  local session="$1"
  local name="$2"
  tmux list-windows -t "$session" -F "#{window_index}:#{window_name}" 2>/dev/null | awk -F: -v n="$name" '$2 == n {print $1; exit}'
}

ensure_window() {
  local session="$1"
  local name="$2"
  local dir="$3"

  if [[ -z "$session" || -z "$name" ]]; then
    return 1
  fi

  local window_index
  window_index=$(find_window_index_by_name "$session" "$name")

  if [[ -z "$window_index" ]]; then
    if [[ -n "$dir" ]]; then
      tmux new-window -d -t "$session" -n "$name" -c "$dir"
    else
      tmux new-window -d -t "$session" -n "$name"
    fi
    window_index=$(find_window_index_by_name "$session" "$name")
  fi

  if [[ -z "$window_index" ]]; then
    return 1
  fi

  printf '%s:%s\n' "$session" "$name"
}

codex_pane_id() {
  local session
  session=$(tmux display-message -p '#S')

  local codex_index
  codex_index=$(find_window_index_by_name "$session" "codex")
  if [[ -z "$codex_index" ]]; then
    return 1
  fi

  tmux list-panes -t "${session}:${codex_index}" -F "#{pane_id}" 2>/dev/null | head -n1
}

codex_repl_running() {
  local pane_id
  if ! pane_id=$(codex_pane_id); then
    return 1
  fi

  local pane_cmd
  pane_cmd=$(tmux display-message -p -t "$pane_id" '#{pane_current_command}' 2>/dev/null || true)

  case "$pane_cmd" in
  codex | launch.sh | codex-wrapper)
    return 0
    ;;
  *)
    return 1
    ;;
  esac
}

# Determine shell init command based on current shell
determine_shell_init() {
  local shell_init=""
  if [[ "$SHELL" == */zsh ]] && [[ -f "$HOME/.zshrc" ]]; then
    shell_init="source ~/.zshrc && "
  elif [[ "$SHELL" == */bash ]] && [[ -f "$HOME/.bashrc" ]]; then
    shell_init="source ~/.bashrc && "
  fi
  echo "$shell_init"
}

# Parse panel arguments (name:command pairs)
# Sets global associative arrays: panel_names, panel_commands
parse_panels() {
  local args=("$@")

  # Skip --panels flag if present
  if [[ "${args[0]:-}" == "--panels" ]]; then
    args=("${args[@]:1}")
  fi

  declare -gA panel_commands=()
  declare -ga panel_names=()

  for arg in "${args[@]}"; do
    # Match pattern: name:command
    if [[ "$arg" =~ ^([a-zA-Z0-9_-]+):(.+)$ ]]; then
      local name="${BASH_REMATCH[1]}"
      local command="${BASH_REMATCH[2]}"

      # Remove quotes if present
      command=$(echo "$command" | sed "s/^['\"]//;s/['\"]$//")

      panel_names+=("$name")
      panel_commands["$name"]="$command"
    else
      log_warn "Ignoring invalid panel format: $arg (expected name:command)"
    fi
  done
}

# Create or attach to leaf session
cmd_init() {
  check_tmux

  local PROJECT_DIR=$(pwd)
  local DIR_NAME=$(basename "$PROJECT_DIR")
  local SESSION_NAME="leaf-$(echo "$DIR_NAME" | sed 's/[^a-zA-Z0-9_-]/_/g')"

  parse_panels "$@"

  if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    log_info "Leaf session '$SESSION_NAME' already exists."
    tmux set-environment -t "$SESSION_NAME" LEAF_PROJECT_ROOT "$PROJECT_DIR" >/dev/null 2>&1 || true
    if [[ -n "${TMUX:-}" ]]; then
      tmux switch-client -t "$SESSION_NAME"
    else
      exec tmux attach -t "$SESSION_NAME"
    fi
    exit 0
  fi

  log_info "Creating leaf session '$SESSION_NAME' in $PROJECT_DIR"

  # Initialize git if not already initialized
  if [[ ! -d "$PROJECT_DIR/.git" ]]; then
    log_info "Initializing git repository..."
    if git -C "$PROJECT_DIR" init >/dev/null 2>&1; then
      log_success "  Git repository initialized"
    else
      log_warn "  Failed to initialize git repository"
    fi
  else
    log_info "Git repository already initialized"
  fi

  # Initialize bd (beads) if not already initialized
  if [[ ! -d "$PROJECT_DIR/.beads" ]]; then
    log_info "Initializing bd (beads) tracker..."
    if (cd "$PROJECT_DIR" && bd init --quiet --skip-merge-driver >/dev/null 2>&1); then
      log_success "  Beads tracker initialized"

      # Install git hooks for bd sync
      if [[ -d "$PROJECT_DIR/.git" ]]; then
        log_info "  Installing bd git hooks..."
        if (cd "$PROJECT_DIR" && bd hooks install >/dev/null 2>&1); then
          log_success "  Git hooks installed"
        else
          log_warn "  Failed to install git hooks"
        fi
      fi
    else
      log_warn "  Failed to initialize beads tracker"
    fi
  else
    log_info "Beads tracker already initialized"
  fi

  # Create CLAUDE.md if it doesn't exist
  if [[ ! -f "$PROJECT_DIR/CLAUDE.md" ]]; then
    log_info "Creating CLAUDE.md starter profile..."
    cat > "$PROJECT_DIR/CLAUDE.md" <<'EOF'
# Project Assistant Profile

You are a highly capable polyglot software engineer and architect with a passion for turning sparks of inspiration into elegant, well-architected solutions.

## Your Role

You excel at:

- **Exploratory Development**: Helping transform early-stage ideas into concrete technical approaches
- **Architecture Guidance**: Suggesting ideal patterns, structures, and tools based on project needs
- **Polyglot Expertise**: Fluent across languages, frameworks, and ecosystems - you find the right tool for the job, not just the familiar one
- **Pragmatic Design**: Balancing ideal architecture with practical constraints and iterative development

## Your Approach

### When Exploring Ideas

- Ask clarifying questions to understand the core problem and constraints
- Suggest multiple viable approaches with trade-offs clearly explained
- Consider both immediate needs and long-term maintainability
- Validate assumptions early and often

### When Choosing Tools

- Evaluate options based on:
  - Problem fit (does this tool solve the actual problem?)
  - Ecosystem maturity (community, libraries, tooling)
  - Developer experience (will this be pleasant to work with?)
  - Performance characteristics (does it meet our needs?)
  - Team expertise (can we maintain this?)

### When Designing Architecture

- Start with the problem domain, not the solution space
- Favor simplicity and clarity over cleverness
- Design for change (assume requirements will evolve)
- Consider operational concerns (deployment, monitoring, debugging)
- Document key decisions and trade-offs

## Development Philosophy

- **Iterate rapidly**: Build the simplest thing that could work, then improve
- **Test ideas**: Prototype before committing to large architectural decisions
- **Stay pragmatic**: Perfect is the enemy of shipped
- **Learn continuously**: Every project teaches something new
- **Communicate clearly**: Code is read more than written

## Issue Tracking with BD (Beads)

This project uses `bd` (beads) for issue tracking with **first-class dependency support**. Issues are chained together like beads, creating a clear dependency graph.

### Core BD Workflow

```bash
# Create issues as you identify work
bd create "Set up project structure"
bd create "Design API schema"
bd create "Implement authentication"

# Add dependencies (blocks/blocked-by relationships)
bd dep add 2 --blocks 3    # Issue 2 blocks issue 3
bd dep add 1 --blocks 2    # Issue 1 blocks issue 2

# See what's ready to work on (no blockers)
bd ready

# Show blocked items
bd blocked

# Update issue status
bd update 1 --status in-progress
bd close 1

# View dependency graph
bd show 3  # Shows all dependencies and blockers
```

### Best Practices for BD

- **Break down work**: Create small, focused issues rather than large epics
- **Express dependencies**: Use `bd dep add` to show what blocks what
- **Track architectural decisions**: Create issues for "Decide on X" or "Research Y"
- **Document blockers**: If something is blocked, make it explicit with dependencies
- **Use labels**: Organize with labels like `architecture`, `research`, `bug`, `feature`
- **Check ready work**: Run `bd ready` frequently to see what can be worked on now
- **Sync with git**: BD hooks automatically sync issues with commits

### BD for Architecture Exploration

When exploring architectural options:

1. Create an issue: `bd create "Research database options for time-series data"`
2. Add comments as you research: `bd comment 1 "PostgreSQL with TimescaleDB looks promising"`
3. Create dependent issues: `bd create "Prototype TimescaleDB integration" && bd dep add 1 --blocks 2`
4. Close research issue when decision is made: `bd close 1`

### BD Tips

- Issues are stored in `.beads/` and tracked in git
- Git hooks keep JSONL files in sync automatically
- Use `bd list` to see all issues
- Use `bd stats` to see project overview
- Use `bd show <id>` to see full issue details with dependencies

## Project Stage

This project is in the **early exploration phase**. The CLAUDE.md file will evolve as the project matures and domain-specific patterns emerge. For now, your role is to help navigate from inspiration to initial implementation with sound architectural foundations.

## Working Together

- Use `bd` proactively to track architectural decisions, technical debt, and implementation tasks
- Break complex problems into manageable issues with clear dependency chains
- Propose experiments when the best path forward is unclear
- Document "why" not just "what" in code and commit messages
- Keep the dependency graph clean - update blockers as work progresses

Let's build something great! 🚀
EOF
    log_success "  CLAUDE.md created"
  else
    log_info "CLAUDE.md already exists"
  fi

  # Create .claude/settings.json if it doesn't exist
  if [[ ! -f "$PROJECT_DIR/.claude/settings.json" ]]; then
    log_info "Creating .claude/settings.json..."
    mkdir -p "$PROJECT_DIR/.claude"
    cat > "$PROJECT_DIR/.claude/settings.json" <<'EOF'
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "model": "sonnet",
  "alwaysThinkingEnabled": true,
  "includeCoAuthoredBy": false,
  "statusLine": {
    "type": "command",
    "command": "~/.claude/statusline-with-bd.sh"
  },
  "companyAnnouncements": [
    "🌱 New Project - Let's explore! Use bd to track ideas and architecture decisions as we build."
  ],
  "permissions": {
    "allow": [
      "Bash(bd:*)"
    ],
    "deny": [],
    "ask": []
  }
}
EOF
    log_success "  .claude/settings.json created"
  else
    log_info ".claude/settings.json already exists"
  fi

  local SHELL_INIT
  SHELL_INIT="$(determine_shell_init)"

  local LOG_CMD="${SHELL_INIT}LOG_FILE=\$HOME/.claude/debug/current.log; mkdir -p \"\$(dirname \"\$LOG_FILE\")\"; touch \"\$LOG_FILE\"; if command -v lnav >/dev/null 2>&1; then lnav \"\$LOG_FILE\"; else tail -n0 -f \"\$LOG_FILE\"; fi"
  local CLAUDE_CMD="${SHELL_INIT}clear && claude"
  local OTEL_CMD="${SHELL_INIT}if command -v otel-tui >/dev/null 2>&1; then CONFIG=\${OTEL_TUI_CONFIG:-\$HOME/.config/otel-tui/config.toml}; if [[ -f \"\$CONFIG\" ]]; then otel-tui --config \"\$CONFIG\"; else echo \"otel-tui config not found at \$CONFIG\"; otel-tui; fi; else echo \"Install otel-tui to visualize Codex telemetry.\"; fi"
  local CODEX_MSG="${SHELL_INIT}printf 'codex window ready. run \"leaf do <task>\" to delegate work to Codex.\\n'"
  local CLIFFY_MSG="${SHELL_INIT}printf 'cliffy window ready. launch cliffy or other sidecar tools here.\\n'"

  tmux new-session -d -s "$SESSION_NAME" -n "coord" -c "$PROJECT_DIR"
  tmux set-environment -t "$SESSION_NAME" LEAF_PROJECT_ROOT "$PROJECT_DIR" >/dev/null 2>&1 || true
  tmux send-keys -t "${SESSION_NAME}:coord" "$CLAUDE_CMD" C-m

  tmux new-window -d -t "$SESSION_NAME" -n "logs" -c "$PROJECT_DIR"
  tmux send-keys -t "${SESSION_NAME}:logs" "$LOG_CMD" C-m

  tmux new-window -d -t "$SESSION_NAME" -n "otel" -c "$PROJECT_DIR"
  tmux send-keys -t "${SESSION_NAME}:otel" "$OTEL_CMD" C-m

  tmux new-window -d -t "$SESSION_NAME" -n "codex" -c "$PROJECT_DIR"
  tmux send-keys -t "${SESSION_NAME}:codex" "$CODEX_MSG" C-m

  tmux new-window -d -t "$SESSION_NAME" -n "cliffy" -c "$PROJECT_DIR"
  tmux send-keys -t "${SESSION_NAME}:cliffy" "$CLIFFY_MSG" C-m

  for panel_name in "${panel_names[@]}"; do
    local panel_cmd="${panel_commands[$panel_name]}"
    local window_idx
    window_idx=$(find_window_index_by_name "$SESSION_NAME" "$panel_name")
    if [[ -z "$window_idx" ]]; then
      tmux new-window -d -t "$SESSION_NAME" -n "$panel_name" -c "$PROJECT_DIR"
    fi
    if [[ -n "$panel_cmd" ]]; then
      tmux send-keys -t "${SESSION_NAME}:$panel_name" C-c
      tmux send-keys -t "${SESSION_NAME}:$panel_name" "${SHELL_INIT}$panel_cmd" C-m
      log_info "  Window '$panel_name': $panel_cmd"
    else
      log_info "  Window '$panel_name' ready (idle shell)."
    fi
  done

  tmux select-window -t "${SESSION_NAME}:coord"

  log_success "Leaf session '$SESSION_NAME' ready!"
  echo ""
  log_info "Windows: coord (Claude), logs, otel, codex, cliffy."
  if [[ ${#panel_names[@]} -gt 0 ]]; then
    log_info "Additional windows configured: ${panel_names[*]}"
  fi
  log_info "Run 'leaf explain' inside the session to see window details."

  if [[ -n "${TMUX:-}" ]]; then
    tmux switch-client -t "$SESSION_NAME"
  else
    exec tmux attach -t "$SESSION_NAME"
  fi
}

# Show context information for current session
cmd_explain() {
  check_tmux

  if [[ -z "${TMUX:-}" ]]; then
    log_error "Not currently in a tmux session."
    echo ""
    echo "Run 'leaf init' from a project directory to create a session."
    exit 1
  fi

  # Get current tmux context
  local CURRENT_SESSION
  local CURRENT_WINDOW
  local CURRENT_PANE
  local CURRENT_PATH
  local CURRENT_CMD
  CURRENT_SESSION=$(tmux display-message -p '#S')
  CURRENT_WINDOW=$(tmux display-message -p '#W')
  CURRENT_PANE=$(tmux display-message -p '#P')
  CURRENT_PATH=$(tmux display-message -p '#{pane_current_path}')
  CURRENT_CMD=$(tmux display-message -p '#{pane_current_command}')
  # Verify we're in a leaf session
  if [[ ! "$CURRENT_SESSION" =~ ^leaf- ]]; then
    log_warn "Not currently in a leaf session (current session: $CURRENT_SESSION)"
    echo "Run 'leaf init' to create a leaf session."
    exit 1
  fi

  echo "=== Leaf Session Context ==="
  echo ""
  echo "Session:  $CURRENT_SESSION"
  echo "Window:   $CURRENT_WINDOW"
  echo "Pane:     $CURRENT_PANE"
  echo "Path:     $CURRENT_PATH"
  echo "Command:  $CURRENT_CMD"
  echo ""

  echo "=== Window Layout ==="
  echo ""
  while IFS=$'\t' read -r win_idx win_name win_active win_width win_height; do
    local marker=""
    if [[ "$win_active" == "1" ]]; then
      marker=" *"
    fi
    echo "  Window $win_idx [$win_name]${marker} (${win_width}x${win_height})"
    while IFS=$'\t' read -r pane_idx pane_title pane_cmd pane_width pane_height; do
      echo "    Pane $pane_idx [$pane_title] (${pane_width}x${pane_height})"
      echo "      Command: $pane_cmd"
    done < <(tmux list-panes -t "${CURRENT_SESSION}:${win_name}" -F "#{pane_index}\t#{pane_title}\t#{pane_current_command}\t#{pane_width}\t#{pane_height}")
    echo ""
  done < <(tmux list-windows -t "$CURRENT_SESSION" -F "#{window_index}\t#{window_name}\t#{window_active}\t#{window_width}\t#{window_height}")

  echo ""
  echo "=== Interacting with Panes ==="
  echo ""
  echo "Send command to an execution pane (pane 1+):"
  echo "  tmux send-keys -t \"$CURRENT_SESSION:0.1\" \"cargo build\" C-m"
  echo ""
  echo "Read output from a pane (last 20 lines):"
  echo "  tmux capture-pane -t \"$CURRENT_SESSION:0.1\" -p -S -20"
  echo ""
  echo "Check what command is running:"
  echo "  tmux display-message -t \"$CURRENT_SESSION:0.1\" -p '#{pane_current_command}'"
  echo ""
  echo "Kill process in a pane (send Ctrl-C):"
  echo "  tmux send-keys -t \"$CURRENT_SESSION:0.1\" C-c"
  echo ""
  echo "=== Context Efficiency Tip ==="
  echo "Instead of running commands inline (consuming full output),"
  echo "send commands to hidden execution panes and read only what you need."
  echo "This saves over 95% of context tokens on long-running processes."
  echo ""
  echo "Example workflow:"
  echo "  1. Start build:  tmux send-keys -t \"$CURRENT_SESSION:0.1\" \"cargo watch\" C-m"
  echo "  2. Check status: tmux capture-pane -t \"$CURRENT_SESSION:0.1\" -p -S -5"
  echo "  3. Read errors:  tmux capture-pane -t \"$CURRENT_SESSION:0.1\" -p | grep error"
}

# Kill current leaf session
cmd_kill() {
  check_tmux

  if [[ -z "${TMUX:-}" ]]; then
    log_error "Not currently in a tmux session."
    exit 1
  fi

  local CURRENT_SESSION=$(tmux display-message -p '#S')

  # Verify we're in a leaf session
  if [[ ! "$CURRENT_SESSION" =~ ^leaf- ]]; then
    log_error "Not currently in a leaf session (current session: $CURRENT_SESSION)"
    echo "Use 'tmux kill-session' for non-leaf sessions."
    exit 1
  fi

  log_info "Killing leaf session '$CURRENT_SESSION'..."
  tmux kill-session -t "$CURRENT_SESSION"
  log_success "Session killed."
}

# List all leaf sessions
cmd_list() {
  check_tmux

  local leaf_sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null | grep "^leaf-" || true)

  if [[ -z "$leaf_sessions" ]]; then
    log_info "No leaf sessions found."
    echo "Run 'leaf init' to create a session."
    exit 0
  fi

  echo "=== Leaf Sessions ==="
  echo ""

  while read -r session; do
    local session_path="unknown"
    if session_path=$(resolve_project_dir "$session"); then
      :
    else
      session_path="unknown"
    fi
    local session_created=$(tmux display-message -t "$session" -p "#{session_created}")
    local window_count=$(tmux list-windows -t "$session" | wc -l | tr -d ' ')

    echo "  $session"
    echo "    Path:     $session_path"
    echo "    Windows:  $window_count"
    echo ""
  done <<<"$leaf_sessions"
}

# Attach to a specific leaf session
cmd_attach() {
  check_tmux

  local session_name="$1"

  if [[ -z "$session_name" ]]; then
    log_error "Session name required."
    echo "Usage: leaf attach <session-name>"
    echo ""
    echo "Available leaf sessions:"
    cmd_list
    exit 1
  fi

  # Ensure session name has leaf- prefix
  if [[ ! "$session_name" =~ ^leaf- ]]; then
    session_name="leaf-$session_name"
  fi

  if ! tmux has-session -t "$session_name" 2>/dev/null; then
    log_error "Session '$session_name' not found."
    exit 1
  fi

  log_info "Attaching to '$session_name'..."

  if [[ -n "${TMUX:-}" ]]; then
    tmux switch-client -t "$session_name"
  else
    exec tmux attach -t "$session_name"
  fi
}

cmd_do_codex() {
  check_tmux

  if [[ -z "${TMUX:-}" ]]; then
    log_error "'leaf do codex' must be run inside a leaf tmux session."
    exit 1
  fi

  local current_session
  current_session=$(tmux display-message -p '#S')
  if [[ ! "$current_session" =~ ^leaf- ]]; then
    log_error "'leaf do codex' only works inside a leaf session (current: $current_session)."
    exit 1
  fi

  local project_dir
  if ! project_dir=$(resolve_project_dir "$current_session"); then
    log_error "Unable to determine project directory for session '$current_session'."
    exit 1
  fi

  local label=""
  local target_window="codex"
  local timeout="900"
  local preview="false"
  local attach="true"
  local capture_lines=0
  local -a extra_args=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
    --pane|--window)
      if [[ -z "${2:-}" ]]; then
        log_error "--pane/--window requires a value."
        exit 1
      fi
      target_window="$2"
      shift 2
      ;;
    --timeout)
      if [[ -z "${2:-}" ]]; then
        log_error "--timeout requires a value (seconds)."
        exit 1
      fi
      timeout="$2"
      shift 2
      ;;
    --preview)
      preview="true"
      shift
      ;;
    --no-attach)
      attach="false"
      shift
      ;;
    --capture)
      if [[ -z "${2:-}" ]]; then
        log_error "--capture requires a value (lines)."
        exit 1
      fi
      capture_lines="$2"
      shift 2
      ;;
    --)
      shift
      while [[ $# -gt 0 ]]; do
        extra_args+=("$1")
        shift
      done
      ;;
    -*)
      extra_args+=("$1")
      shift
      ;;
    *)
      if [[ -z "$label" ]]; then
        label="$1"
      else
        extra_args+=("$1")
      fi
      shift
      ;;
    esac
  done

  if [[ -z "$label" ]]; then
    log_error "'leaf do codex' requires a task label."
    echo "Usage: leaf do codex <label> [--pane pane] [--timeout seconds] [--preview] [--no-attach] [--capture N]"
    exit 1
  fi

  local label_slug
  label_slug=$(slugify "$label")

  local plans_dir="$project_dir/.leaf/plans"
  local history_dir="$project_dir/.leaf/history"
  local runs_dir="$project_dir/.leaf/runs"
  local status_dir="$project_dir/.leaf/status"

  mkdir -p "$history_dir" "$runs_dir" "$status_dir" "$plans_dir"

  local plan_file=""
  if [[ -f "$plans_dir/${label}.yml" ]]; then
    plan_file="$plans_dir/${label}.yml"
  elif [[ -f "$plans_dir/${label_slug}.yml" ]]; then
    plan_file="$plans_dir/${label_slug}.yml"
  fi

  local plan_codex_args=""
  local plan_title="$label"
  local plan_instructions=""

  if [[ -n "$plan_file" ]]; then
    local title_line
    title_line=$(grep -E '^title:' "$plan_file" | head -n1 || true)
    if [[ -n "$title_line" ]]; then
      plan_title=$(echo "$title_line" | sed 's/^title:[[:space:]]*//')
    fi

    local codex_args_line
    codex_args_line=$(grep -E '^codex_args:' "$plan_file" | head -n1 || true)
    if [[ -n "$codex_args_line" ]]; then
      plan_codex_args=$(echo "$codex_args_line" | sed 's/^codex_args:[[:space:]]*//')
    fi

    plan_instructions=$(awk '
      /^instructions:/ {
        line=$0
        sub(/^instructions:[[:space:]]*/, "", line)
        if (line != "" && line !~ /^[|>]/) {
          print line
          exit
        }
        capture=1
        next
      }
      capture == 1 {
        if ($0 !~ /^[[:space:]]+/ && $0 !~ /^$/) {
          exit
        }
        sub(/^[[:space:]]*/, "")
        print
      }
    ' "$plan_file")
  else
    log_warn "Plan file not found for '$label'; proceeding with defaults."
  fi

  local target_window_index
  target_window_index=$(find_window_index_by_name "$current_session" "$target_window")
  if [[ -z "$target_window_index" ]]; then
    log_warn "Window '$target_window' not found; creating new execution window."
    tmux new-window -d -t "$current_session" -n "$target_window" -c "$project_dir"
  fi

  local target_ref="${current_session}:${target_window}"
  local status_window="leaf-do-status"
  local status_window_index
  status_window_index=$(find_window_index_by_name "$current_session" "$status_window")
  if [[ -z "$status_window_index" ]]; then
    tmux new-window -d -t "$current_session" -n "$status_window" -c "$project_dir"
  fi
  local status_ref="${current_session}:${status_window}"

  tmux send-keys -t "$status_ref" C-c

  local label_escaped
  label_escaped=$(printf "%s" "$plan_title" | sed "s/'/'\"'\"'/g")

  local run_id
  run_id=$(date +%Y%m%d%H%M%S)
  local run_suffix
  printf -v run_suffix "%04d" $((RANDOM % 10000))
  local run_key="${run_id}-${run_suffix}"

  local status_file="$history_dir/${run_key}-${label_slug}.log"
  : >"$status_file"
  ln -sf "$status_file" "$history_dir/latest-${label_slug}.log"

  tmux send-keys -t "$status_ref" "clear" C-m
  tmux send-keys -t "$status_ref" "printf 'leaf do codex :: ${label_escaped}\n\n'" C-m
  tmux send-keys -t "$status_ref" "tail -n 200 -f '$status_file'" C-m

  local instructions_file=""
  if [[ -n "$plan_instructions" ]]; then
    instructions_file="$runs_dir/${run_key}-${label_slug}-instructions.txt"
    printf "%s\n" "$plan_instructions" >"$instructions_file"
  fi

  local -a final_args=()
  if [[ -n "$plan_codex_args" ]]; then
    # shellcheck disable=SC2206
    local plan_args_array=($plan_codex_args)
    final_args+=("${plan_args_array[@]}")
  fi
  if [[ ${#extra_args[@]} -gt 0 ]]; then
    final_args+=("${extra_args[@]}")
  fi

  local run_script="$runs_dir/${run_key}-${label_slug}.sh"
  local summary_file="$history_dir/${run_key}-${label_slug}-summary.md"
  local exit_code_file="$status_dir/${run_key}-${label_slug}.exit"
  local wait_key="leaf-plan-${run_key}"

  local status_file_q
  local wait_key_q
  local label_q
  local label_slug_q
  local run_key_q
  local project_dir_q
  local instructions_file_q
  local summary_file_q
  local exit_code_file_q
  local timeout_q
  local preview_flag="$preview"

  printf -v status_file_q '%q' "$status_file"
  printf -v wait_key_q '%q' "$wait_key"
  printf -v label_q '%q' "$plan_title"
  printf -v label_slug_q '%q' "$label_slug"
  printf -v run_key_q '%q' "$run_key"
  printf -v project_dir_q '%q' "$project_dir"
  printf -v summary_file_q '%q' "$summary_file"
  printf -v exit_code_file_q '%q' "$exit_code_file"
  timeout_q="$timeout"

  if [[ -n "$instructions_file" ]]; then
    printf -v instructions_file_q '%q' "$instructions_file"
  else
    instructions_file_q="\"\""
  fi

  printf '%s\n' "#!/usr/bin/env bash" >"$run_script"
  cat <<EOF >>"$run_script"
set -euo pipefail

STATUS_FILE=$status_file_q
WAIT_KEY=$wait_key_q
RUN_LABEL=$label_q
RUN_SLUG=$label_slug_q
RUN_KEY=$run_key_q
PROJECT_ROOT=$project_dir_q
INSTRUCTIONS_FILE=$instructions_file_q
SUMMARY_FILE=$summary_file_q
EXIT_CODE_FILE=$exit_code_file_q
TIMEOUT=$timeout_q
PREVIEW=$preview_flag
LAUNCH_SCRIPT="\$PROJECT_ROOT/.leaf/launch.sh"

mkdir -p "\$(dirname "\$STATUS_FILE")" "\$(dirname "\$SUMMARY_FILE")" "\$(dirname "\$EXIT_CODE_FILE")"

exec > >(stdbuf -oL tee -a "\$STATUS_FILE") 2>&1

START_TS=\$(date -Iseconds)
echo "[leaf do codex] Task: \$RUN_LABEL"
echo "[leaf do codex] Run ID: \$RUN_KEY"
echo "[leaf do codex] Started: \$START_TS"

if [[ -n "\$INSTRUCTIONS_FILE" && -f "\$INSTRUCTIONS_FILE" ]]; then
  echo "[leaf do codex] Instructions:"
  sed 's/^/  /' "\$INSTRUCTIONS_FILE"
fi

if [[ -z "\${CODEX_CONFIG_PATH:-}" && -f "\$PROJECT_ROOT/.leaf/codex.toml" ]]; then
  export CODEX_CONFIG_PATH="\$PROJECT_ROOT/.leaf/codex.toml"
fi

if [[ ! -x "\$LAUNCH_SCRIPT" ]]; then
  echo "[leaf do codex] Missing executable launch script at \$LAUNCH_SCRIPT" >&2
  exit 1
fi

CMD=()
CMD+=("\$LAUNCH_SCRIPT")
CMD+=("codex" "exec")
if [[ -n "\${CODEX_CONFIG_PATH:-}" ]]; then
  CMD+=("--config" "\$CODEX_CONFIG_PATH")
fi
CMD+=("-C" "\$PROJECT_ROOT")
CMD+=("--sandbox" "workspace-write")
CMD+=("-c" "logging.level=debug")
CMD+=("-c" "telemetry.otel.enabled=true")
CMD+=("-c" "telemetry.otel.exporter=debug")
if [[ -n "\$INSTRUCTIONS_FILE" && -f "\$INSTRUCTIONS_FILE" ]]; then
  CMD+=("-")
fi
EOF

  if [[ ${#final_args[@]} -gt 0 ]]; then
    local arg_line=""
    for arg in "${final_args[@]}"; do
      printf -v arg_line 'CMD+=(%q)\n' "$arg"
      printf '%s' "$arg_line" >>"$run_script"
    done
  fi

  cat <<'EOF' >>"$run_script"
if [[ "$PREVIEW" == "true" ]]; then
  echo "[leaf do codex] Preview mode enabled (command not executed)."
  printf '[leaf do codex] Command:'
  printf ' %q' "${CMD[@]}"
  printf '\n'
  if [[ -n "${TMUX_PANE:-}" ]]; then
    tmux pipe-pane -t "${TMUX_PANE}"
  fi
  tmux wait-for -S "$WAIT_KEY"
  exit 0
fi

exit_code=0
if [[ -n "$INSTRUCTIONS_FILE" && -f "$INSTRUCTIONS_FILE" ]]; then
  "${CMD[@]}" < "$INSTRUCTIONS_FILE" || exit_code=$?
else
  "${CMD[@]}" || exit_code=$?
fi

END_TS=$(date -Iseconds)
echo "[leaf do codex] Finished: $END_TS (exit $exit_code)"

if [[ -n "${TMUX_PANE:-}" ]]; then
  tmux pipe-pane -t "${TMUX_PANE}"
fi

if [[ -n "$SUMMARY_FILE" ]]; then
  {
    echo "# leaf do codex :: $RUN_LABEL"
    echo "- Run ID: $RUN_KEY"
    echo "- Started: $START_TS"
    echo "- Finished: $END_TS"
    echo "- Exit Code: $exit_code"
    echo ""
    echo "## Instructions"
    if [[ -n "$INSTRUCTIONS_FILE" && -f "$INSTRUCTIONS_FILE" ]]; then
      cat "$INSTRUCTIONS_FILE"
    else
      echo "(none provided)"
    fi
    echo ""
    echo "## Command"
    printf '`'
    printf '%q ' "${CMD[@]}"
    printf '`\n'
  } >"$SUMMARY_FILE"
fi

echo "$exit_code" >"$EXIT_CODE_FILE"
tmux wait-for -S "$WAIT_KEY"
exit "$exit_code"
EOF

  chmod +x "$run_script"

  tmux pipe-pane -t "$target_ref"
  tmux pipe-pane -t "$target_ref" -o "stdbuf -oL tee -a '$status_file'"

  local shell_init
  shell_init=$(determine_shell_init)

  tmux send-keys -t "$target_ref" C-c
  tmux send-keys -t "$target_ref" "cd $(printf '%q' "$project_dir")" C-m
  tmux send-keys -t "$target_ref" "${shell_init}bash $(printf '%q' "$run_script")" C-m

  if [[ "$attach" == "true" ]]; then
    tmux wait-for -L "$wait_key"
    tmux pipe-pane -t "$target_ref"
    local exit_code="0"
    if [[ -f "$exit_code_file" ]]; then
      exit_code=$(cat "$exit_code_file")
    fi
    if [[ "$capture_lines" -gt 0 ]]; then
      echo ""
      echo "=== leaf do codex :: last ${capture_lines} line(s) ==="
      tail -n "$capture_lines" "$status_file" || true
      echo "=== end capture ==="
    fi
    echo ""
    echo "leaf do codex :: task '${plan_title}' completed with exit code $exit_code"
    echo "status log: $status_file"
    echo "summary:    $summary_file"
    exit "$exit_code"
  else
    echo "leaf do codex :: task '${plan_title}' running in window '$target_window' (status: $status_file)"
    echo "Use 'tmux wait-for -L $wait_key' later to wait for completion if needed."
  fi
}

cmd_plan() {
  check_tmux

  if [[ -z "${TMUX:-}" ]]; then
    log_error "'leaf plan' must be run inside a leaf tmux session."
    exit 1
  fi

  local current_session
  current_session=$(tmux display-message -p '#S')
  if [[ ! "$current_session" =~ ^leaf- ]]; then
    log_error "'leaf plan' only works inside a leaf session (current: $current_session)."
    exit 1
  fi

  local project_dir
  if ! project_dir=$(resolve_project_dir "$current_session"); then
    log_error "Unable to determine project directory for session '$current_session'."
    exit 1
  fi

  local resume_id=""
  local action=""
  local -a action_args=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
    --resume)
      if [[ -z "${2:-}" ]]; then
        log_error "--resume requires a value."
        exit 1
      fi
      resume_id="$2"
      shift 2
      ;;
    attach | send)
      action="$1"
      shift
      action_args=("$@")
      break
      ;;
    *)
      action="$1"
      shift
      action_args=("$@")
      break
      ;;
    esac
  done

  if [[ -z "$action" ]]; then
    action="attach"
  fi

  if [[ "$action" != "attach" && "$action" != "send" ]]; then
    log_error "Unknown 'leaf plan' action: $action"
    echo ""
    echo "Usage:"
    echo "  leaf plan [--resume id]         # Start or attach Codex REPL"
    echo "  leaf plan attach [--resume id]  # Alias for attach"
    echo "  leaf plan send \"message\"      # Send prompt to REPL"
    exit 1
  fi

  local codex_ref
  if ! codex_ref=$(ensure_window "$current_session" "codex" "$project_dir"); then
    log_error "Unable to locate or create 'codex' window."
    exit 1
  fi

  local -f start_codex_repl ensure_codex_repl

  start_codex_repl() {
    local pane_id
    if ! pane_id=$(codex_pane_id); then
      return 1
    fi

    tmux pipe-pane -t "$pane_id"
    tmux send-keys -t "$pane_id" C-c
    tmux send-keys -t "$pane_id" "clear" C-m
    tmux send-keys -t "$pane_id" "cd $(printf '%q' "$project_dir")" C-m

    local shell_init
    shell_init=$(determine_shell_init)

    local -a repl_cmd=(".leaf/launch.sh" "codex" "exec" "--interactive")
    if [[ -n "$resume_id" ]]; then
      repl_cmd+=("--resume" "$resume_id")
    fi
    local repl_cmd_str=""
    printf -v repl_cmd_str '%q ' "${repl_cmd[@]}"

    tmux send-keys -t "$pane_id" "${shell_init}${repl_cmd_str}" C-m

    local resume_note=""
    if [[ -n "$resume_id" ]]; then
      resume_note=" (resume: $resume_id)"
    fi
    tmux display-message -t "$codex_ref" "leaf plan :: Codex REPL launched${resume_note}"
    log_info "Codex REPL launch command sent${resume_note}."
  }

  ensure_codex_repl() {
    if codex_repl_running; then
      return 0
    fi

    log_warn "Codex REPL not running; launching interactive session..."
    if ! start_codex_repl; then
      log_error "Failed to start Codex REPL."
      exit 1
    fi
    sleep 0.2
  }

  case "$action" in
  attach)
    if [[ ${#action_args[@]} -gt 0 ]]; then
      log_error "'leaf plan attach' does not accept additional arguments."
      exit 1
    fi
    ensure_codex_repl
    local pane_id
    if ! pane_id=$(codex_pane_id); then
      log_error "Unable to select Codex pane."
      exit 1
    fi
    tmux select-window -t "$codex_ref"
    tmux select-pane -t "$pane_id"
    log_success "Attached to Codex REPL (window 'codex')."
    ;;
  send)
    local prompt=""
    if [[ ${#action_args[@]} -eq 0 ]]; then
      log_error "'leaf plan send' requires a prompt string."
      exit 1
    fi
    prompt="${action_args[*]}"

    ensure_codex_repl

    local pane_id
    if ! pane_id=$(codex_pane_id); then
      log_error "Unable to locate Codex pane."
      exit 1
    fi

    tmux send-keys -t "$pane_id" -l -- "$prompt"
    tmux send-keys -t "$pane_id" C-m
    tmux display-message -t "$codex_ref" "leaf plan :: prompt sent"
    log_success "Sent prompt to Codex REPL."
    ;;
  esac
}

cmd_status() {
  check_tmux

  if [[ -z "${TMUX:-}" ]]; then
    log_error "'leaf status' must be run inside a tmux session."
    exit 1
  fi

  local action="${1:-}"
  local current_session
  current_session=$(tmux display-message -p '#S')

  case "$action" in
  clean)
    local plan_window_index
    plan_window_index=$(find_window_index_by_name "$current_session" "leaf-plan-status")
    if [[ -n "$plan_window_index" ]]; then
      tmux kill-window -t "${current_session}:leaf-plan-status"
      log_success "Leaf plan status window removed."
    fi

    local do_window_index
    do_window_index=$(find_window_index_by_name "$current_session" "leaf-do-status")
    if [[ -n "$do_window_index" ]]; then
      tmux kill-window -t "${current_session}:leaf-do-status"
      log_success "Leaf do status window removed."
    fi

    if [[ -z "$plan_window_index" && -z "$do_window_index" ]]; then
      log_info "No status windows found."
    fi
    ;;
  *)
    log_error "Usage: leaf status clean"
    exit 1
    ;;
  esac
}

cmd_do() {
  check_tmux

  if [[ -z "${TMUX:-}" ]]; then
    log_error "'leaf do' must be run inside a leaf tmux session."
    exit 1
  fi

  local subcommand="${1:-}"
  if [[ -z "$subcommand" ]]; then
    log_error "'leaf do' requires a subcommand."
    echo ""
    echo "Available subcommands:"
    echo "  leaf do codex <task> [options]   Run Codex plan YAML"
    echo "  leaf do tasks [options]          Execute batch tasks via cliffy"
    echo ""
    echo "Future subcommands:"
    echo "  leaf do commit             Create git commit"
    echo "  leaf do release            Manage release workflow"
    exit 1
  fi

  shift

  case "$subcommand" in
  codex)
    cmd_do_codex "$@"
    ;;
  tasks)
    cmd_do_tasks "$@"
    ;;
  *)
    log_error "Unknown 'leaf do' subcommand: $subcommand"
    echo ""
    echo "Available subcommands:"
    echo "  codex    Run Codex plan YAML"
    echo "  tasks    Execute batch tasks via cliffy"
    exit 1
    ;;
  esac
}

cmd_do_tasks() {
  local current_session
  current_session=$(tmux display-message -p '#S')

  if [[ ! "$current_session" =~ ^leaf- ]]; then
    log_error "'leaf do tasks' only works inside a leaf session (current: $current_session)."
    exit 1
  fi

  local project_dir
  if ! project_dir=$(resolve_project_dir "$current_session"); then
    log_error "Unable to determine project directory for session '$current_session'."
    exit 1
  fi

  local target_window="cliffy"
  local tasks_file=""
  local context_file=""
  local attach="true"
  local max_concurrent=0
  local preset=""
  local output_format="text"
  local -a cliffy_args=()
  local -a task_args=()

  # Parse options
  while [[ $# -gt 0 ]]; do
    case "$1" in
    --tasks-file)
      if [[ -z "${2:-}" ]]; then
        log_error "--tasks-file requires a value."
        exit 1
      fi
      tasks_file="$2"
      shift 2
      ;;
    --context-file)
      if [[ -z "${2:-}" ]]; then
        log_error "--context-file requires a value."
        exit 1
      fi
      context_file="$2"
      shift 2
      ;;
    --max-concurrent)
      if [[ -z "${2:-}" ]]; then
        log_error "--max-concurrent requires a value."
        exit 1
      fi
      max_concurrent="$2"
      shift 2
      ;;
    --preset)
      if [[ -z "${2:-}" ]]; then
        log_error "--preset requires a value."
        exit 1
      fi
      preset="$2"
      shift 2
      ;;
    --output-format)
      if [[ -z "${2:-}" ]]; then
        log_error "--output-format requires a value."
        exit 1
      fi
      output_format="$2"
      shift 2
      ;;
    --window)
      if [[ -z "${2:-}" ]]; then
        log_error "--window requires a value."
        exit 1
      fi
      target_window="$2"
      shift 2
      ;;
    --no-attach)
      attach="false"
      shift
      ;;
    -*)
      cliffy_args+=("$1")
      shift
      ;;
    *)
      task_args+=("$1")
      shift
      ;;
    esac
  done

  # Ensure cliffy window exists
  local target_window_index
  target_window_index=$(find_window_index_by_name "$current_session" "$target_window")
  if [[ -z "$target_window_index" ]]; then
    log_warn "Window '$target_window' not found; creating it."
    tmux new-window -d -t "$current_session" -n "$target_window" -c "$project_dir"
  fi

  local target_ref="${current_session}:${target_window}"
  local status_window="leaf-do-status"
  local status_window_index
  status_window_index=$(find_window_index_by_name "$current_session" "$status_window")
  if [[ -z "$status_window_index" ]]; then
    tmux new-window -d -t "$current_session" -n "$status_window" -c "$project_dir"
  fi
  local status_ref="${current_session}:${status_window}"

  # Build cliffy command
  local -a cmd=("cliffy" "exec")

  if [[ -n "$tasks_file" ]]; then
    cmd+=("--tasks-file" "$tasks_file")
  fi

  if [[ -n "$context_file" ]]; then
    cmd+=("--context-file" "$context_file")
  fi

  if [[ "$max_concurrent" -gt 0 ]]; then
    cmd+=("--max-concurrent" "$max_concurrent")
  fi

  if [[ -n "$preset" ]]; then
    cmd+=("--preset" "$preset")
  fi

  cmd+=("--output-format" "$output_format")
  cmd+=("${cliffy_args[@]}")
  cmd+=("${task_args[@]}")

  # Create run ID for logging
  local run_id
  run_id=$(date +%Y%m%d%H%M%S)
  local run_suffix
  printf -v run_suffix "%04d" $((RANDOM % 10000))
  local run_key="${run_id}-${run_suffix}"

  local history_dir="$project_dir/.leaf/history"
  mkdir -p "$history_dir"
  local status_file="$history_dir/${run_key}-tasks.log"
  : >"$status_file"

  # Setup status window
  tmux send-keys -t "$status_ref" C-c
  tmux send-keys -t "$status_ref" "clear" C-m
  tmux send-keys -t "$status_ref" "printf 'leaf do tasks :: running cliffy batch\n\n'" C-m
  tmux send-keys -t "$status_ref" "tail -n 200 -f '$status_file'" C-m

  # Setup pipe and execute
  tmux pipe-pane -t "$target_ref"
  tmux pipe-pane -t "$target_ref" -o "stdbuf -oL tee -a '$status_file'"

  local shell_init
  shell_init=$(determine_shell_init)

  tmux send-keys -t "$target_ref" C-c
  tmux send-keys -t "$target_ref" "cd $(printf '%q' "$project_dir")" C-m

  local cmd_str=""
  printf -v cmd_str '%q ' "${cmd[@]}"
  tmux send-keys -t "$target_ref" "${shell_init}${cmd_str}" C-m

  if [[ "$attach" == "true" ]]; then
    log_info "Waiting for cliffy to complete..."
    log_info "Output streaming to: $status_file"
    log_info "Switch to window '$status_window' to monitor progress."
  else
    log_info "Cliffy running in window '$target_window' (background mode)"
    log_info "Status log: $status_file"
    log_info "Monitor progress in window '$status_window'"
  fi
}

# Main command dispatcher
main() {
  local cmd="${1:-}"

  case "$cmd" in
  init)
    shift
    cmd_init "$@"
    ;;
  plan)
    shift
    cmd_plan "$@"
    ;;
  do)
    shift
    cmd_do "$@"
    ;;
  explain)
    cmd_explain
    ;;
  kill)
    cmd_kill
    ;;
  list | ls)
    cmd_list
    ;;
  attach | a)
    shift
    cmd_attach "$@"
    ;;
  status)
    shift
    cmd_status "$@"
    ;;
  help | --help | -h)
    usage
    ;;
  version | --version | -v)
    echo "Leaf v${VERSION}"
    ;;
  "")
    log_error "No command specified."
    echo ""
    usage
    exit 1
    ;;
  *)
    log_error "Unknown command: $cmd"
    echo ""
    usage
    exit 1
    ;;
  esac
}

main "$@"
