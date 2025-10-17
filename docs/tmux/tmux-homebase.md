# tmux Home Base

Use this repo as a jump-off point for running your AI coding assistants in persistent `tmux` sessions. The `scripts/start_tmux_homebase.sh` script will create a ready-to-go session that keeps Claude Code, Codex, and your task runner alive even when you detach from the terminal.

## Prerequisites

- Install tmux: `brew install tmux` on macOS, `apt install tmux` on Debian/Ubuntu, or `yum install tmux` on RHEL/CentOS.
- Ensure the helper commands you plan to run (web servers, CLIs, task runners) are available on `PATH`.

## Quick Start

```sh
# Launch the session with default commands
./scripts/start_tmux_homebase.sh
tmux attach -t home-base
```

Built-in defaults:

- Claude window runs `claude`
- Codex window runs `codex`
- Tasks window runs `tb`

Override any of them by exporting `TMUX_CLAUDE_CMD`, `TMUX_CODEX_CMD`, or `TMUX_TASK_CMD` before running the script.

Detach at any time with `Ctrl-b` followed by `d`. Your processes keep running until you re-attach or stop the session.

## Session Layout

The script creates a session named `home-base` (override with `TMUX_SESSION_NAME`) with three windows:

| Window | Default command (override with env var) | Typical use |
| ------ | --------------------------------------- | ----------- |
| `claude` | `claude` (override: `TMUX_CLAUDE_CMD`) | Claude Code CLI for AI-assisted coding |
| `codex` | `codex` (override: `TMUX_CODEX_CMD`) | Codex CLI for codebase exploration and analysis |
| `tasks` | `tb` (override: `TMUX_TASK_CMD`) | Toolbox task runner for automated workflows |

If a command is not found in your PATH, the window opens with a shell prompt and displays a warning.

## Customising Windows

- **Different commands:** export `TMUX_CLAUDE_CMD`, `TMUX_CODEX_CMD`, or `TMUX_TASK_CMD` before running the script.
  - Example: `TMUX_CLAUDE_CMD="claude --project myapp" ./scripts/start_tmux_homebase.sh`
- **Rename the session:** set `TMUX_SESSION_NAME=my-session ./scripts/start_tmux_homebase.sh`.
- **Add more windows:** edit `scripts/start_tmux_homebase.sh` and append to the `window_names` / `window_cmds` arrays. Use the existing entries as a template.
- **Run once:** the script exits early if the session already exists so you can safely re-run it from shell start-up scripts without duplicating sessions.

## Day-to-Day Workflow

- List sessions: `tmux ls`
- Attach: `tmux attach -t home-base`
- Detach: `Ctrl-b` then `d`
- Rename a window: `Ctrl-b` then `,`
- Split panes: `Ctrl-b` then `%` (vertical) or `Ctrl-b` then `"` (horizontal)
- Pane navigation: `Ctrl-b` + arrow keys
- Kill a window or pane: `Ctrl-b` then `&`
- Kill the whole session: `tmux kill-session -t home-base`

## Tips

- Use `Ctrl-b` then `:` to open the tmux command prompt for ad-hoc commands (for example `rename-session` or `rename-window`).
- Copy mode: `Ctrl-b` then `[` lets you scroll back or yank text with `Space` (start) and `Enter` (copy).
- If a command crashes, the pane stays open; use the arrow keys to inspect logs and press `Up` to re-run it.
- Keep the script under version control so changes to window layout are recorded alongside other project automation.

Feel free to create additional scripts for specific AI tooling setups or development stacks and document them alongside this guide as your home base grows.
