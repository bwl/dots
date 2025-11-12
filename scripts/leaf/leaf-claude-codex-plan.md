# Claude ↔︎ Codex Per-Project Plan

This plan captures how each project workspace should be wired so Claude acts as the coordinator, while Codex and other CLIs handle execution inside a Leaf-managed tmux session.

## Operating Principles
- **Delegate execution**: Claude only orchestrates by handing work to `codex exec`, `cliffy`, and other approved commands. All heavy coding, testing, and research stays in those tools.
- **Minimize context churn**: Run noisy commands in hidden panes and surface only summarized output back to Claude.
- **Observability first**: Keep OpenTelemetry (OTel) export enabled in Codex with an always-on `otel-tui` window for live traces and spans.
- **Per-project isolation**: Each repository gets its own config bundle (`.leaf/`) so swapping projects preserves command allowlists, environment variables, and tmux layout.

## Session Layout
| Window | Purpose | Commands |
| --- | --- | --- |
| `coord` | Claude Code interactive shell for planning and high-level control. | `claude` (with project profile) |
| `codex` | Dedicated execution lane for Codex CLI tasks. | `codex exec --config $PROJECT_ROOT/.leaf/codex.toml` |
| `cliffy` | Secondary agent or tooling runner (Cliffy, MCP tools, etc.). | `cliffy` or scripted commands |
| `otel` | OpenTelemetry dashboard. | `otel-tui --config ~/.config/otel-tui/config.toml` |
| `logs` | Tail important project logs or Claude debug output. | `lnav ~/.claude/debug/current.log` |

All windows live inside the tmux session named `leaf-<project-name>` so Claude can switch targets and sample output without losing context. Each window hosts a single pane by default; `leaf do` creates an additional `leaf-do-status` window whenever tasks run.

## Command Control Model
1. Claude issues natural-language instructions.
2. Local scripts translate approved intents into shell commands routed to `codex exec` or other tools.
3. Codex performs edits/tests with `otel` debug enabled for each invocation.
4. Summaries or captured output are piped back to Claude through minimal snippets (e.g., `tmux capture-pane -S -100`).

Use a narrow allowlist (`.leaf/allowlist.yml`) describing which commands Claude can trigger automatically. Require human confirmation for anything outside the list.

## Observability Thread
- Codex configuration sets `telemetry.otel.enabled = true` with `exporter = "debug"` so spans emit locally.
- `otel-collector` (optional) can fan out to remote backends; default setup targets loopback.
- `otel-tui` runs pinned in the `otel` window for real-time inspection. When needed, export traces to file (`otel-tui --record traces.json`).

## Workflow Loop
1. Human opens project and runs `leaf sync` (scripted wrapper described in setup doc) to rebuild the tmux layout.
2. Claude bootstraps: reads `.leaf/brief.md`, checks outstanding tasks, queues Codex jobs.
3. Codex executes, instrumentation appears in `otel` window, results summarized back to Claude.
4. Human reviews spans or logs before approving deployments or merges.

## Next Steps
The accompanying setup guide (`leaf-claude-codex-setup.md`) describes how to implement this plan, provision configs, and automate the tmux session creation workflow.
