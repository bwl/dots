# Concept: `leaf do` Wrapper for `codex exec`

`leaf do` extends Leaf with a task-oriented command that routes Claude’s requests through Codex in a safe, observable, and context-efficient way. Claude already excels at high-level planning and command orchestration inside tmux.citeturn1open0 `leaf do` gives Codex a first-class lane for edits, testing, and research while Leaf manages windows, approvals, and telemetry.

## Goals
- **Single entry point**: Claude runs `leaf do build`, `leaf do fix-tests`, etc., instead of calling `codex exec` directly.
- **Opinionated defaults**: Always run Codex with project-scoped config, verbose logging, and OTEL debug exporters so spans land in the `otel` window.citeturn0open0
- **Human-trust loop**: Leaf keeps humans in the loop with status windows, summarized output, and command approval workflows.
- **Capability visibility**: Surface Codex’s strengths—multi-file edits, automated tests, refactors—so Claude is confident delegating large tasks.

## CLI Shape
```
leaf do <label> [--window codex] [--timeout 900] [--preview] [--no-attach] [--capture N]
```

- `<label>` maps to a task profile stored in `.leaf/tasks/<label>.yml` (instructions + default Codex flags). Hyphenated slugs (e.g. `plan-clean-architecture`) are supported; copy the example from `task-examples/plan-clean-architecture.yml`.
- `--window` chooses the execution window (defaults to `codex`; auto-creates it if missing). `--pane` remains accepted as an alias.
- `--timeout` forwards an execution timeout to Codex (default 900 seconds).
- `--preview` prints the fully expanded Codex command to the status log without executing it.
- `--no-attach` returns immediately instead of waiting for Codex to finish; output continues streaming into the status window.
- `--capture N` prints the last `N` log lines to the invoking terminal after the run concludes.
- Additional CLI arguments are passed straight through to `codex exec`.

## Codex Invocation Template
`leaf do` expands to:
```
.leaf/launch.sh codex exec \
  --config "$CODEX_CONFIG_PATH" \
  --label "<label>" \
  --workspace "$PROJECT_ROOT" \
  --otel-debug \
  --log-level debug \
  --sandbox project \
  --timeout <seconds> \
  [extra flags from task profile]
```

Recommended defaults (all supported by the Codex config surface):citeturn0open0
- `--config`: ensures consistent logging + OTEL exporters.
- `--workspace`: pins Codex to the project root so relative paths match Leaf windows.
- `--otel-debug`: keeps spans flowing into `otel-tui` even if a remote exporter is configured.
- `--log-level debug`: mirrors `logging.level = "debug"` for alignment between CLI and config file output.
- `--sandbox project`: scopes file access to the repo, aligning with Leaf’s per-project isolation.
- `--timeout`: prevents runaway tasks and allows Leaf to notify the human if Codex stalled.
- Add your own approval hooks by wrapping Codex or using `.leaf/allowlist.yml`; the core wrapper keeps the surface flexible.

## Runtime Experience
- **Persistent status window**: `leaf do` ensures there is a dedicated `leaf-do-status` window that tails the run log in real time. The window stays open so Claude or a human can review the output; remove it later with `leaf status clean`.
- **Trace view**: Because Codex executes with OTEL debug enabled, spans land in the pre-existing `otel` window where otel-tui is running.
- **Result capture**: Each run writes both a streaming log (`.leaf/history/<timestamp>-<label>.log`) and a summary markdown report (`.leaf/history/<timestamp>-<label>-summary.md`) describing start/end timestamps, exit code, and referenced instructions.
- **Exit codes and captures**: Leaf stores the exit code under `.leaf/status/<timestamp>-<label>.exit`. When `--capture N` is supplied, the invoking terminal prints the tail of the run so Claude can summarize without additional `tmux` calls.
- **Cleanup hook**: `leaf status clean` kills the status window once everyone has reviewed it, without disturbing other window layouts.

## Feature Opportunities
- **Adaptive prompts**: Prepend task briefs (from `.leaf/tasks`) and latest history so Codex operates with rich project context.
- **Chained tasks**: Allow `leaf do` to accept pipelines (`leaf do refactor | leaf do test`) where the output summary of one becomes context for the next.
- **Human checkpoint**: Auto-open Claude’s window with a yes/no prompt whenever Codex proposes destructive changes outside the allowlist.
- **Telemetry bookmarks**: Write span IDs to `.leaf/history` so humans can jump directly to the related trace in `otel-tui`.
- **Capability manifest**: During `leaf do --preview`, print a checklist of Codex powers (multi-file edits, test execution, doc lookup) to remind Claude what it can safely delegate this run.

## Why Codex Shines Here
- Codex is designed for end-to-end coding cycles: plan, edit, test, and iterate across complex repositories.
- It understands multi-file architecture, can apply patches atomically, and respects tool-specific workflows (e.g., `cargo`, `npm`, `pytest`) without overwhelming Claude’s context window.
- With OTEL instrumentation enabled, Codex exposes granular spans for planning, edits, diffing, and validation, giving humans observability that pure AI-to-terminal flows lack.

By baking these behaviors into `leaf do`, Claude gains a trustworthy automation lever: invoke a single command, rely on Codex for the heavy lifting, and stay informed through Leaf’s windows, summaries, and telemetry.
