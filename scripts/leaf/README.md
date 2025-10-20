# Leaf - AI-Optimized Tmux Workspace

**Leaf** is a standalone CLI tool for creating context-efficient tmux workspaces designed specifically for AI coding agents like Claude Code.

## Philosophy

Traditional terminal workflows waste AI context on verbose build output, test results, and server logs. Leaf implements the **selective attention pattern**:

- Long-running processes execute in hidden, named panes
- AI agents sample output only when needed (last 5-20 lines)
- **Saves 80-90% of context tokens** compared to inline execution

Instead of this (traditional):
```bash
npm run build  # 5,000 lines → 15,000 tokens consumed
```

Do this (leaf):
```bash
# Send to hidden pane
tmux send-keys -t "leaf-proj:0.1" "npm run build" C-m

# Later, check status (last 5 lines only)
tmux capture-pane -t "leaf-proj:0.1" -p -S -5  # 200 tokens
```

See `~/dotfiles/claude-thoughts/09-multiplexer-as-ai-agent-interface.md` for the conceptual framework.

## Claude ↔ Codex ↔ Cliffy Workflow

Leaf provides three main execution modes:

1. **`leaf plan`** - Persistent Codex REPL for interactive planning and follow-ups
2. **`leaf do codex`** - One-off Codex runs driven by YAML tasks
3. **`leaf do tasks`** - Claude delegates batch tasks to Cliffy for parallel execution
4. **Manual windows** - Direct interaction with build/test/serve processes

Documentation:
- `leaf-claude-codex-plan.md` outlines the per-project workflow where Claude coordinates Codex, Cliffy, and observability windows
- `leaf-claude-codex-setup.md` provides step-by-step instructions for wiring Codex OTEL telemetry, otel-tui, and command allowlists inside Leaf sessions
- `leaf-do-codex-wrapper.md` explores the `leaf do codex` command that wraps `codex exec` with status windows, telemetry defaults, and task profiles

## Installation

Leaf is installed at `~/bin/leaf` (symlink to `~/dotfiles/scripts/leaf/leaf.sh`).

```bash
# Create symlink (if not already created)
ln -sf ~/dotfiles/scripts/leaf/leaf.sh ~/bin/leaf

# Verify installation
leaf version
```

### Dependencies

**Required:**
- `tmux` - Terminal multiplexer (core dependency)
- `codex` - Required for `leaf do` command

**Optional:**
- `lnav` - Enhanced log viewer (falls back to `tail -f` if unavailable)
- `otel-tui` - OpenTelemetry visualization (shows reminder message if not installed)

**Per-Project Setup (for `leaf do`):**

Each project needs a `.leaf/` directory with:

```bash
# Copy templates from dotfiles
mkdir -p ~/Developer/my-project/.leaf
cp ~/dotfiles/scripts/leaf/templates/launch.sh ~/Developer/my-project/.leaf/
cp ~/dotfiles/scripts/leaf/templates/codex.toml ~/Developer/my-project/.leaf/

# Ensure launch.sh is executable
chmod +x ~/Developer/my-project/.leaf/launch.sh
```

The `launch.sh` wrapper sets environment variables for Codex. The `codex.toml` is optional - Codex will use global defaults if not present.

## Quick Start

### Default Layout - Claude, Codex, and Telemetry

```bash
cd ~/Developer/my-project
leaf init

# Creates a telemetry-aware tmux session with dedicated windows:
#   [0] coord   → Claude Code (active)
#   [1] logs    → lnav/tail following ~/.claude/debug/current.log
#   [2] otel    → otel-tui (or a reminder if unavailable)
#   [3] codex   → Codex REPL controlled by `leaf plan`
#   [4] cliffy  → execution lane for `leaf do tasks` (Cliffy batch tasks)
#   leaf-do-status   → created on demand during `leaf do codex` / `leaf do tasks` runs
#
# After reviewing tasks, clean up status windows with:
#   leaf status clean
```

**Features:**
- **Coord window**: Claude interactive session stays focused on orchestration and summaries
- **Logs window**: lnav (or `tail -f`) watches Claude debug output without blowing up context
- **Otel window**: `otel-tui` visualizes Codex spans as soon as telemetry arrives
- **Codex window**: Interactive Codex REPL managed by `leaf plan`
- **Cliffy window**: Execution lane for `leaf do tasks` - parallel batch tasks with Cliffy
- **Status windows**: `leaf do codex` / `leaf do tasks` tail output persistently for audit; clean up with `leaf status clean`

### With Execution Windows

```bash
cd ~/Developer/rust-project
leaf init build:'cargo watch -x build' test:'cargo test' serve:'cargo run'

# Additional windows:
#   [5] build → cargo watch -x build
#   [6] test  → cargo test
#   [7] serve → cargo run
# Default windows (coord/logs/otel/codex/cliffy) remain available.
```

### Web Development Example

```bash
cd ~/Developer/web-app
leaf init \
  build:'npm run build:watch' \
  test:'npm run test:watch' \
  serve:'npm run dev' \
  db:'docker-compose up postgres'

# Each task spawns its own window alongside the defaults.
# Switch temporarily (prefix + number) or use `tmux capture-pane` to sample output.
```

## Commands

### `leaf init [panel:command ...]`

Create or attach to a leaf session for the current directory.

**Arguments:**
- `panel:command` - Optional named execution windows (see Window Spec Syntax below)

**Examples:**
```bash
# Just Claude
leaf init

# With panels
leaf init build:'make watch' test:'pytest --watch'

# Complex setup
leaf init \
  build:'cargo watch -x build -x test' \
  serve:'cargo run --bin server' \
  logs:'tail -f /var/log/app.log'
```

**Behavior:**
- Creates session named `leaf-<directory-name>`
- If session exists, attaches to it (safe to run multiple times)
- All windows start in the current project directory
- **Default windows:** `coord`, `logs`, `otel`, `codex`, `cliffy` (all created detached except `coord`)
- Extra `panel:command` arguments add or update windows with matching names
- `leaf do codex` and `leaf do tasks` stream output to `leaf-do-status` window
- Remove status windows with `leaf status clean`

### `leaf plan` (Interactive Codex REPL)

`leaf plan` manages a persistent Codex REPL in the `codex` window so you can keep architectural conversations alive without reloading context.

**Usage:**
- `leaf plan` or `leaf plan attach` &mdash; focus the Codex REPL (launches automatically if it is not running)
- `leaf plan send "<message>"` &mdash; send a prompt to the REPL (text send followed by a separate `C-m`)
- `leaf plan --resume <id>` &mdash; pass a Codex resume token when the REPL is relaunched

**Behavior:**
- Ensures the `codex` window exists and starts `.leaf/launch.sh codex exec --interactive`
- Clears the pane, kills stale pipe-pane output, and sends `C-c` before each launch
- Auto-relaunches whenever the REPL is inactive; `leaf plan send` will start it on demand
- Emits tmux status messages and CLI logs so coordinators know when the REPL restarts

**Tips:**
- Use $'multi\nline' quoting to preserve newlines in prompts.
- `leaf plan send` queues two tmux calls (text then `C-m`) to avoid the race we saw with combined sends.

**Prerequisites:** Requires `.leaf/launch.sh` in your project root (see Per-Project Setup above). The script must support `codex exec --interactive`.

### `leaf do codex <task> [options]`

Run Codex plans from `.leaf/plans/<task>.yml` as one-off executions with telemetry enabled and status streaming.

**Prerequisites:** Same `.leaf/launch.sh` wrapper used for the REPL.

- Plans live in `.leaf/plans/<task>.yml` (see `plan-examples/plan-clean-architecture.yml` and `plan-examples/fibonacci-funny.yml`)
- `leaf do codex clean-architecture --capture 40` runs Codex with OTEL debug enabled, streams output to `.leaf/history`, and prints the last 40 lines locally
- `--no-attach` allows long Codex jobs to continue in the background; monitor progress in the persistent `leaf-do-status` window and close it later with `leaf status clean`

**Options:**
- `--window <name>` - Target window (default: codex)
- `--timeout <seconds>` - Execution timeout (default: 900)
- `--preview` - Show command without executing
- `--no-attach` - Run in background
- `--capture <N>` - Print last N lines after completion

### `leaf do tasks [options] [task1] [task2] ...`

Execute batch tasks via Cliffy for parallel execution of simple, independent tasks.

**Use Cases:**
- Code analysis: `leaf do tasks "analyze auth.go" "review db.go" "check tests"`
- Documentation: `leaf do tasks "document API" "update README" "check TODOs"`
- Quick fixes: `leaf do tasks "fix typos in docs" "update dependencies"`

**Options:**
- `--tasks-file <file>` - Read tasks from file (one per line)
- `--context-file <file>` - Shared context for all tasks
- `--max-concurrent <N>` - Max parallel tasks (default: 3)
- `--preset <name>` - Use Cliffy preset (e.g., sec-review, fast-qa)
- `--output-format <format>` - Output format: text, json, diff
- `--window <name>` - Target window (default: cliffy)
- `--no-attach` - Run in background

**Examples:**
```bash
# Inline tasks
leaf do tasks "list Go files" "count TODO comments" "check license headers"

# From file with context
echo "review auth.go
audit payments.go
scan session.go" > security-tasks.txt
leaf do tasks --tasks-file security-tasks.txt --context-file security-rules.md --preset sec-review

# Parallel analysis
leaf do tasks --max-concurrent 5 "analyze file1.go" "analyze file2.go" "analyze file3.go"
```

### `leaf explain`

Show context information for the current leaf session. **Run this inside a tmux session.**

**Output:**
- Current session, window, and active pane info
- Window roster with pane titles and running commands
- Examples of interacting with windows programmatically
- Context efficiency tips

**Example (abridged):**
```
=== Leaf Session Context ===
Session:  leaf-myproject
Window:   coord
Pane:     0
Path:     /Users/bwl/Developer/my-project
Command:  claude

=== Window Layout ===
  Window 0 [coord] * (211x54)
    Pane 0 [coord] (211x54)
      Command: claude

  Window 1 [logs] (211x54)
    Pane 0 [logs] (211x54)
      Command: lnav

  Window 2 [otel] (211x54)
    Pane 0 [otel] (211x54)
      Command: otel-tui --config ...
...
```

### `leaf kill`

Kill the current leaf session. **Must be run inside a leaf tmux session.**

```bash
# Inside a leaf session
leaf kill

# Kills the session and exits tmux
```

### `leaf list`

List all active leaf sessions.

```bash
leaf list

# Output:
# === Leaf Sessions ===
#
#   leaf-dotfiles
#     Path:     /Users/bwl/dotfiles
#     Windows:  5
#
#   leaf-web-app
#     Path:     /Users/bwl/Developer/web-app
#     Windows:  7
```

### `leaf attach <session>`

Attach to a specific leaf session by name.

```bash
# Attach by full session name
leaf attach leaf-dotfiles

# Or by project name (leaf- prefix added automatically)
leaf attach dotfiles

# From within another tmux session, switches to the target session
```

### `leaf help`

Show usage information and examples.

### `leaf version`

Show leaf version.

## Window Spec Syntax

Window specs are provided as `name:command` pairs separated by spaces (the CLI flag remains `panel:command` for backward compatibility).

**Format:** `name:command`

- **Name**: Alphanumeric, dashes, underscores (e.g., `build`, `test-unit`, `serve_dev`)
- **Command**: Any shell command (use quotes if it contains spaces or special characters)

**Examples:**
```bash
# Simple commands
leaf init build:'make' test:'pytest'

# Commands with arguments
leaf init build:'cargo watch -x build' test:'npm run test:watch'

# Complex commands with pipes/redirects
leaf init log:'tail -f app.log | grep ERROR' serve:'PORT=3000 npm start'

# Multiple word commands (quotes required)
leaf init serve:'bun run --hot dev' db:'docker-compose up -d postgres redis'
```

**Window names** appear in tmux’s status line, making it easy for Claude or a human to navigate by name.

## Workflow Patterns

### For Users

```bash
# Create session for a project
cd ~/Developer/my-app
leaf init build:'npm run build' test:'npm test' serve:'npm run dev'

# Work in Claude pane (you're already there)
# Claude can monitor the hidden panes programmatically

# List sessions
leaf list

# Switch to another project
leaf attach other-project

# Kill current session when done
leaf kill
```

### For Claude Code (AI Agent)

When working in a leaf session, use `leaf explain` to discover panes, then interact with them:

```bash
# Note: Pane 0 = logs, Pane 1 = claude, Pane 2+ = execution panes

# Start a build in the build pane (first execution pane = pane 2)
tmux send-keys -t "leaf-myproject:0.2" "npm run build" C-m

# Check if build completed (last 3 lines)
tmux capture-pane -t "leaf-myproject:0.2" -p -S -3

# Look for success/failure
tmux capture-pane -t "leaf-myproject:0.2" -p | tail -5 | grep -q "Build succeeded"

# Read full test output from test pane (pane 3)
tmux capture-pane -t "leaf-myproject:0.3" -p -S -50

# Kill a process in serve pane (pane 4)
tmux send-keys -t "leaf-myproject:0.4" C-c

# Restart with new command
tmux send-keys -t "leaf-myproject:0.4" "cargo build --release" C-m

# View debug logs in real-time (they're in pane 0)
# Just look at the top pane, or zoom it:
tmux select-pane -t "leaf-myproject:0.0"
tmux resize-pane -Z  # Toggle zoom
```

### Context Efficiency Comparison

**Traditional inline execution:**
```
Run cargo build          → 30,000 tokens (full output)
Fix error
Run cargo build again    → 30,000 tokens
Fix error
Run cargo build again    → 30,000 tokens
─────────────────────────────────────────
Total: 90,000 tokens for 3 builds
```

**Leaf with background monitoring:**
```
Start cargo watch in execution pane (pane 2)
Check last 5 lines       → 200 tokens
Fix error
Check last 5 lines       → 200 tokens
Fix error
Check last 5 lines       → 200 tokens
─────────────────────────────────────────
Total: 600 tokens for continuous builds
```

**150x more efficient!**

## Advanced Usage

### State Persistence

Hidden panes maintain state across AI agent interactions:
- Working directory stays set
- Environment variables persist
- Virtual environments remain activated
- Background processes continue running

**Example:**
```bash
# Initial setup in a panel
tmux send-keys -t "leaf-app:0.2" "cd backend" C-m
tmux send-keys -t "leaf-app:0.2" "source venv/bin/activate" C-m
tmux send-keys -t "leaf-app:0.2" "export DEBUG=1" C-m
tmux send-keys -t "leaf-app:0.2" "python manage.py runserver" C-m

# Later, the environment is still there
tmux send-keys -t "leaf-app:0.2" C-c  # Stop server
tmux send-keys -t "leaf-app:0.2" "python manage.py migrate" C-m  # Still in venv, has DEBUG
```

### Parallel Execution

Run multiple processes simultaneously:

```bash
leaf init \
  build:'cargo watch -x build' \
  test:'cargo watch -x test' \
  serve:'cargo run' \
  db:'docker-compose up postgres'

# All 4 processes run in parallel
# Claude monitors them asynchronously
# No blocking, no context explosion
```

### Project-Specific Aliases

Create shell aliases for common setups:

```bash
# In ~/.zshrc or ~/.bashrc

# Rust project
alias leaf-rust="leaf init build:'cargo watch -x build' test:'cargo watch -x test' serve:'cargo run'"

# Node.js project
alias leaf-node="leaf init build:'npm run build:watch' test:'npm test -- --watch' serve:'npm run dev'"

# Full-stack app
alias leaf-fullstack="leaf init \
  be-build:'cd backend && npm run build:watch' \
  fe-build:'cd frontend && npm run build:watch' \
  be-serve:'cd backend && npm run dev' \
  fe-serve:'cd frontend && npm run dev' \
  db:'docker-compose up postgres redis'"
```

Then:
```bash
cd ~/Developer/my-rust-project
leaf-rust  # Instant setup!
```

### Integration with Taskbook

While leaf doesn't include a tasks sidebar by default, you can add one as a panel:

```bash
leaf init \
  build:'cargo watch' \
  test:'cargo test' \
  tasks:'watch -n 2 "tb --storage-dir ./.taskbook"'

# Or use your own task tracking command
leaf init tasks:'watch -n 1 "todo.txt list"'
```

## Configuration

### Environment Variables

- **`LEAF_CLAUDE_CMD`** - Command to run in the main Claude pane (default: `claude --debug`)

**Example:**
```bash
export LEAF_CLAUDE_CMD="claude --debug --model sonnet-4.5"
leaf init

# Claude pane will run: claude --debug --model sonnet-4.5
# Debug output automatically saved to ~/.claude/debug/UUID.txt
```

**Note:** The `--debug` flag is included by default to enable debug logging. You can override this, but debug logs won't be captured if you remove `--debug`.

### Debug Logs

Claude automatically manages debug logs when run with `--debug` flag:

**Location:** `~/.claude/debug/UUID.txt` (managed by Claude)

**Features:**
- Real-time viewing in logs window (lnav or tail -f watching `current.log`)
- Each session gets a unique UUID.txt file
- `~/.claude/debug/latest` symlink points to current session
- Leaf uses `current.log` (empty file that you can redirect debug output to if needed)

**Managing logs:**
```bash
# List all debug logs
ls -la ~/.claude/debug/

# View this session's log
readlink ~/.claude/debug/latest  # Get the actual file
lnav $(readlink ~/.claude/debug/latest)

# Clean up old logs (manual)
cd ~/.claude/debug && rm -f $(ls -t *.txt | tail -n +50)  # Keep last 50 logs

# Claude manages these logs, no need to gitignore (not in project directory)
```

### Tmux Status Bar

To show pane titles in your tmux status bar, add to `~/.tmux.conf`:

```tmux
# Show pane title in border
set -g pane-border-format "#{pane_index}:#{pane_title}"
set -g pane-border-status top

# Or show in status bar
set -g status-right "Panes: #{window_panes} | Current: #{pane_title}"
```

## Troubleshooting

**Panels aren't hidden:**
- They're minimized to 1 line but still visible at the bottom
- Use `Ctrl-b z` to zoom the Claude pane to full-screen (hides others completely)
- Or manually resize: `tmux resize-pane -t 0 -y 999`

**Commands in panels don't have access to aliases:**
- Leaf sources `~/.zshrc` or `~/.bashrc` before running commands
- If still not working, use full paths: `/usr/local/bin/npm` instead of `npm`

**Panel command parsing issues:**
- Always quote commands with spaces: `build:'cargo watch -x build'`
- Escape special characters: `log:'tail -f app.log | grep "ERROR"'`
- For complex commands, consider using a script: `build:'./scripts/watch.sh'`

**Session name conflicts:**
- Leaf creates sessions named `leaf-<directory>`
- If you have multiple projects with the same directory name, sessions will clash
- Workaround: Create unique project directories or manually name sessions

**Want to add panels to existing session:**
- Leaf doesn't support adding panels after creation yet
- Workaround: Kill session and recreate with new panels
- Or manually create panes with tmux split-window

## Comparison with Fresh

Leaf is completely separate from the `fresh` command (multi-window tmux launcher for multiple AI agents).

| Feature | Leaf | Fresh |
|---------|------|-------|
| **Purpose** | Single-agent context efficiency | Multi-agent collaboration |
| **Structure** | 1 window, multiple hidden panes | 5 windows (claude, codex, cliffy, git, tasks) |
| **Default** | Just Claude pane | Claude + Codex + Cliffy + Lazygit + Tasks |
| **Customization** | CLI args for execution panes | Environment variables for commands |
| **Best for** | Solo Claude work, long-running processes | Multi-tool workflows, visual git operations |
| **Context savings** | 80-90% on background processes | Standard inline execution |

**When to use:**
- Use **leaf** for focused Claude work with maximum context efficiency
- Use **fresh** for multi-agent workflows with different tools running simultaneously

## Examples by Language/Stack

### Rust

```bash
leaf init \
  build:'cargo watch -x build' \
  test:'cargo watch -x test' \
  clippy:'cargo watch -x clippy' \
  serve:'cargo run'
```

### Node.js/TypeScript

```bash
leaf init \
  build:'tsc --watch' \
  test:'vitest --watch' \
  lint:'eslint --watch' \
  serve:'npm run dev'
```

### Python

```bash
leaf init \
  test:'pytest-watch' \
  serve:'python manage.py runserver' \
  celery:'celery -A myapp worker' \
  db:'docker-compose up postgres'
```

### Go

```bash
leaf init \
  build:'air'  # Live reload tool \
  test:'go test ./... -watch' \
  serve:'go run cmd/server/main.go' \
  db:'docker-compose up postgres redis'
```

### Full-Stack (React + FastAPI)

```bash
leaf init \
  fe-build:'cd frontend && npm run dev' \
  be-serve:'cd backend && uvicorn main:app --reload' \
  be-test:'cd backend && pytest --watch' \
  db:'docker-compose up postgres redis' \
  logs:'tail -f backend/logs/app.log'
```

## Command Summary

| Command | Purpose | Window | Status Window |
|---------|---------|--------|---------------|
| `leaf plan` | Interactive Codex REPL controller | codex | - |
| `leaf do codex <task>` | Codex YAML execution | codex | leaf-do-status |
| `leaf do tasks <tasks>` | Batch execution via Cliffy | cliffy | leaf-do-status |
| `leaf init [windows]` | Create/attach session | - | - |
| `leaf status clean` | Remove status windows | - | - |

## See Also

- **Conceptual framework**: `~/dotfiles/claude-thoughts/09-multiplexer-as-ai-agent-interface.md`
- **Fresh (multi-window mode)**: `~/dotfiles/scripts/tmux-fresh/README.md`
- **Tmux documentation**: `man tmux` or https://github.com/tmux/tmux/wiki
- **Cliffy documentation**: `~/Developer/cliffy/TLDR.CLIFFY.TXT`

## Contributing

Leaf is part of the `~/dotfiles` repository. To modify:

1. Edit `~/dotfiles/scripts/leaf/leaf.sh`
2. Test changes: `~/dotfiles/scripts/leaf/leaf.sh init`
3. Commit changes: `cd ~/dotfiles && git add scripts/leaf && git commit`

## License

Part of personal dotfiles - use freely.
