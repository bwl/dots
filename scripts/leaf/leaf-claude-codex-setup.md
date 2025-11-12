# Claude ↔︎ Codex Leaf Setup Guide

This guide turns the plan into concrete steps so every project launches a Leaf tmux session where Claude coordinates, Codex executes, and observability stays live.

## 1. Prerequisites
- **Claude Code CLI** with project profiles configured, so Claude can orchestrate terminal actions.
- **Codex CLI** installed and authenticated; supports a per-project config file and OTEL exporters.
- **otel-tui** built from source or installed via your preferred package manager for visualizing OpenTelemetry traces locally.
- Optional: Local OTLP collector (e.g., `otelcol`) if you want to forward traces beyond the debug exporter.

## 2. Project Layout
Create a `.leaf/` directory in each repository to hold automation glue:

```
.leaf/
  codex.toml          # Codex config with logging + OTEL enabled
  allowlist.yml       # Commands Claude may trigger automatically
  brief.md            # Human-readable project state snapshot for Claude
  launch.sh           # Wrapper that exports env vars then calls leaf
```

`brief.md` seeds Claude with project context on attach (open it manually in the `coord` window at session start).

## 3. Codex Configuration
Populate `.leaf/codex.toml` and point `CODEX_CONFIG_PATH` at it (see §4).

```toml
# .leaf/codex.toml
root = "."

[logging]
level = "debug"
format = "compact"

[telemetry]
otel = { enabled = true, exporter = "debug", service_name = "codex-${PROJECT_NAME}" }
```

Key points from the Codex config spec: `format` toggles between `json` and `compact`, and the OTEL block enables span export with the built-in debug exporter. The same section can target a remote collector by setting `endpoint`, `headers`, or credentials.

## 4. Environment Bootstrap
Use `.leaf/launch.sh` (invoked from the `codex` window) to keep variables consistent:

```bash
#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CODEX_CONFIG_PATH="$PROJECT_ROOT/.leaf/codex.toml"
export CODEX_LOG_FORMAT=compact
export CODEX_OTEL_DEBUG=1          # forces debug spans even if exporter flips
export OTEL_EXPORTER_OTLP_ENDPOINT="${OTEL_EXPORTER_OTLP_ENDPOINT:-http://127.0.0.1:4317}"

exec "$@"
```

Ensure the script is executable (`chmod +x .leaf/launch.sh`). Leaf windows call it via `.leaf/launch.sh codex exec ...`.

## 5. Leaf Session Recipe
`leaf init` now launches the default `logs`, `coord`, `otel`, `codex`, and `cliffy` windows. Add a helper script or make target when you need to append more automation lanes:

```bash
# ./scripts/leaf-sync.sh
#!/usr/bin/env bash
set -euo pipefail

leaf init \
  codex:'.leaf/launch.sh codex exec --interactive' \
  cliffy:'.leaf/launch.sh cliffy --profile codex-sidekick' \
  otel:'.leaf/launch.sh otel-tui --config ~/.config/otel-tui/config.toml' \
  logs:'lnav ~/.claude/debug/current.log'
```

- Passing `panel:command` pairs that match the default window names updates the running commands instead of creating duplicates.

- Pane `coord` (default) runs `claude` with your project profile.
- `codex` executes edits/tests; OTEL debug spans stream to the exporter configured above.
- `cliffy` offers a second automation lane (CLI summarizer, MCP tool runner, etc.).
- `otel` keeps `otel-tui` in watch mode; switch to it when you need to drill into spans.
- `logs` tails Claude’s own debug output for cross-checking orchestrator activity.


## 6. Command Allowlists
Define guardrails in `.leaf/allowlist.yml` to keep Claude restricted to known-safe commands:

```yaml
allowed:
  - name: run_build
    windows: [codex]
    command: "cargo build"
  - name: run_tests
    windows: [codex]
    command: "cargo test --all-targets"
  - name: fetch_docs
    windows: [cliffy]
    command: "cliffy research --source internal-wiki"
requires_confirmation:
  - name: apply_migrations
    command: "diesel migration run"
```

Integrate validation in your Claude prompt or wrapper so any command outside `allowed` pauses for human approval.

## 7. otel-tui Usage
`otel-tui` reads spans from the OTLP endpoint and renders aggregates, traces, and attribute filters inside the terminal UI. Use hotkeys (`s` for spans, `t` for traces, `:` for command palette) to slice recent Codex runs.

For long investigations, record the session:

```bash
otel-tui --record traces-${PROJECT_NAME}-$(date +%Y%m%d%H%M).json
```

## 8. Daily Workflow
1. Run `./scripts/leaf-sync.sh` from the project root.
2. Claude reads `.leaf/brief.md`, aligns tasks, and keeps the `coord` window focused on planning.
3. Delegate work: Claude triggers allowlisted commands, Codex performs edits/tests, spans stream to `otel`.
4. Sample output via `tmux capture-pane` when ready to summarize back to Claude.
5. Before merge/deploy, review the latest spans in `otel` and logs in `logs` to confirm there are no hidden failures.

## 9. Future Enhancements
- Auto-open the `otel` window on errors by subscribing to Codex span events.
- Add a `leaf audit` command that compiles OTEL traces + command history for compliance.
- Feed `otel-tui --record` output into Claude’s context when debugging intermittent issues.
