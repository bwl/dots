# Setup Home Base

This workspace provides the `hearth` command - a project-specific tmux session launcher for AI coding assistants. Run `hearth` from any project directory to spawn Claude Code, Codex, and tb in that project's context.

## Installation

The `hearth` command is installed globally at `~/bin/hearth` (already in your PATH).

## Quick Start

```bash
# Navigate to your project
cd ~/Developer/my-project

# Launch AI assistants and enter the session
hearth

# You're now in the session! Run 'hearth' in this directory again to rejoin.
```

**Note:** For a context-efficient single-window alternative, see the standalone `leaf` command (`~/dotfiles/scripts/leaf/README.md`).

## What You Get

Each `hearth` session creates five windows in your project directory:
- **claude** (0) - Claude Code CLI for AI-assisted coding
- **codex** (1) - Codex CLI for codebase exploration and analysis
- **cliffy** (2) - Non-interactive LLM task runner with CLI interface
- **git** (3) - Lazygit TUI for visual git operations
- **tasks** (4) - Toolbox (tb) task runner for project workflows

All windows start in the project directory, so all agents work in your project's context automatically.

### Layout

Agent windows (claude, codex, cliffy, git) include a **tasks sidebar** on the right:
- **Left pane (70%)**: The agent/tool interface where you interact
- **Right pane (30%)**: Auto-rehearthing task list (updates every 2 seconds)

The tasks window itself is full-screen for detailed task management. Switch between windows with:
- `Ctrl-b n` - next window
- `Ctrl-b p` - previous window
- `Ctrl-b 0-4` - jump to specific window

## Workflow

```bash
# Each project directory gets its own session
cd ~/work/api && hearth        # Creates/joins 'api' session
cd ~/personal/website && hearth # Creates/joins 'website' session

# List all sessions
tmux ls

# Return to a project session from anywhere
cd ~/work/api && hearth        # Back to 'api' session

# Kill the current session
hearth kill                    # Kills current session and exits tmux

# Or detach with Ctrl-b then d to keep agents running in the background
```

## For Coding Agents

When running inside a tmux session, agents can discover their context:

```bash
hearth explain
```

This shows:
- Current session, window, and pane information
- Project directory path
- All available windows in the session
- Code examples for sending commands to other panes
- Code examples for reading output from other panes

Agents should run `hearth explain` to understand their tmux environment before attempting to interact with other windows.

## Customization

Override default commands via environment variables:
- `TMUX_CLAUDE_CMD` (default: `claude`)
- `TMUX_CODEX_CMD` (default: `codex`)
- `TMUX_CLIFFY_CMD` (default: `cliffy`)
- `TMUX_GIT_CMD` (default: `lazygit`)
- `TMUX_TASK_CMD` (default: `tb`)

## Documentation

- `docs/tmux-homebase.md` - Human-focused tmux guide with keyboard shortcuts
- `docs/tmux-agent-commands.md` - Technical reference for agents to interact with tmux programmatically
