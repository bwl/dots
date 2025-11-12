# leaf plan :: Refactor leaf plan to Use Interactive Codex REPL
- Run ID: 20251019191802-1551
- Started: 2025-10-19T19:18:04-04:00
- Finished: 2025-10-19T19:20:09-04:00
- Exit Code: 0

## Instructions
You are Codex collaborating with Claude. Claude is running low on context tokens
and needs your help refactoring the leaf tool.

## Current State
- `leaf plan <task>` runs one-off `codex exec` commands using YAML task files
- Every execution re-contextualizes from scratch (inefficient)
- Task files are in `.leaf/plans/<task>.yml`
- Implementation is in `leaf.sh` starting at line 412 (`cmd_plan` function)

## Desired State
- `leaf plan` → Starts/attaches to persistent Codex interactive REPL in codex window
- `leaf plan send "<prompt>"` → Sends prompt to running REPL via tmux send-keys
- `leaf do codex <task>` → One-off `codex exec` for batch tasks (moves current logic)

## Your Tasks
1. Read and analyze `leaf.sh` focusing on:
- `cmd_plan()` function (line 412+)
- `cmd_do()` router and `cmd_do_tasks()` as examples
- How tmux window management works

2. Design the new architecture:
- New `cmd_plan()` for interactive REPL management
* `leaf plan` with no args: start/attach Codex REPL in codex window
* `leaf plan send "<prompt>"`: send prompt to REPL via tmux
* Detect if REPL already running (check process in codex window)
- New `cmd_do_codex()` for one-off exec tasks
* Move current `cmd_plan` logic here
* Keep YAML task file support
* Use `leaf-do-status` window (like `cmd_do_tasks`)

3. Provide implementation plan with:
- Exact code changes needed
- How to handle REPL session lifecycle
- How to detect/attach to existing REPL
- Updated help text
- Migration notes for existing `.leaf/plans/*.yml` files

4. Consider edge cases:
- REPL crashes/exits
- Multiple prompts sent rapidly
- How to view REPL output from coord window
- Resume support (`codex resume`)

Focus on clean, maintainable code that follows the existing leaf patterns.

## Command
`/Users/bwl/dotfiles/scripts/leaf/.leaf/launch.sh codex exec -c logging.level=debug -c sandbox.default_mode=workspace-write -c telemetry.otel.enabled=true -c telemetry.otel.exporter=debug `
