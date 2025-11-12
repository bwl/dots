# Leaf Observability & Time Intelligence Review

## Executive Summary
- The existing three-layer proposal provides a solid security boundary but needs clearer contracts, shared data models, and failure-handling semantics to avoid brittle tmux coupling.
- Heuristic state detection is viable for the near term; however, it must be modular, continuously calibrated, and backed by capture samples plus regression tests to keep accuracy above 90%.
- A lightweight telemetry pipeline (append-only JSONL → optional SQLite/duckDB compaction) should underpin execution-time metrics, anomaly alerts, and predictive estimates.
- Early investment in a state/event cache service (daemonized or on-demand) will curb tmux pressure, enable smart waits, and centralize deduped logging for downstream features.
- Prioritize deliverables that unblock raw tmux blacklisting (status/peek/wait + logging) before expanding into daemonization, notifications, and predictive intelligence.

## Detailed Architectural Analysis

### Three-Layer Command Surface
- **Layer contracts**: Commands → detection → tmux abstraction is sound, but the plan needs explicit APIs (input schema, error codes, caching hints) so commands can evolve without re-reading plan text.
- **Session awareness**: Current sketch assumes a single `leaf-leaf` session; commands should accept a session context (default to active) to support parallel workstreams and future remote use.
- **Error propagation**: Define structured failures (pane missing, capture truncated, low confidence) so layer 1 can render friendly guidance instead of surfacing raw tmux stderr.
- **Extensibility**: Keep layer 3 limited to a `tmux` shim module with trace logging; this makes swapping tmux for another multiplexer or remote execution wrapper feasible later.

### Command and UX Layer
- `leaf status|peek|tail|watch|wait` cover primary use cases; add a `--pane-id` option for automation scripts that already know tmux IDs and a `--since` flag for time-based queries.
- Provide machine-readable output by default (`--format text|json|table`) so Claude can rely on structured fields without fragile parsing.
- Smart waits should emit periodic progress snapshots (state, elapsed, predicted remaining) derived from the metrics layer to give Claude confidence during long-running actions.

### State Detection Engine
- **Heuristics**: Split rules per agent into declarative configs (YAML/JSON) that map regexes to score contributions; ship with defaults and allow customization under `.leaf/detect/*.yml`.
- **Confidence bands**: Publish thresholds for high/medium/low confidence and expose them in the JSON output; commands can downgrade to `unknown` instead of misclassifying low-signal panes.
- **Sampling**: Support variable capture depths (last 10/50/100 lines) and incorporate metadata like tmux activity timestamp to distinguish stale panes from live output.
- **Regression harness**: Collect labeled captures (plan already hints at this) and run them through CI to guard against heuristic regressions when prompt formats evolve.
- **Fallback logic**: When heuristics collide (e.g., prompt visible while a stack trace scrolls), prefer state transitions that respect recency and last-known-state; otherwise surface `ambiguous`.

### tmux Abstraction Layer
- Centralize tmux calls in a small library that enforces rate limits, retries transient failures (`pane unknown`), and masks private environment data before logging.
- Guard against panes with massive history by capping capture size and optionally piping through `tail -n` before heuristics.
- Add tracing hooks (`LEAF_DEBUG_TMUX=1`) to log command latency and errors, which will be crucial during observability rollout.

### Caching & Performance
- Cache should be keyed by session/window/pane and contain the capture excerpt, detected state, confidence, last update, and hash of the heuristic config used.
- Instead of TTL-only invalidation, also expire on detected output deltas (compare capture hash) and any send-keys/write events.
- For multi-command bursts (`leaf watch`), reuse cached captures but re-run heuristics with fresh timestamps to keep elapsed time accurate.
- Consider a small daemon (Phase 3+) that owns the cache and pushes updates via a Unix socket/JSON-RPC API; CLI commands become thin clients, reducing tmux load.

### Logging & Storage
- State transitions, command invocations, and heuristic diagnostics should flow into `.leaf/logs/executions.jsonl`. Use one JSON object per event with ISO timestamps and UUID correlation IDs.
- Provide a compaction cron (`leaf metrics compact`) that writes summarized aggregates into SQLite or DuckDB for fast historical queries without bloating JSONL.
- Store capture samples separately (`.leaf/samples/<agent>/<state>/<timestamp>.txt`) with references in the JSONL to reproduce classification decisions.

## Execution Time Tracking & Prediction Layer
- **Event model**: Emit events for `state.enter`, `state.exit`, `task.start`, `task.finish`, `command.wait`, and `anomaly.detected`. Each event should include `agent`, `window`, `task_id` (if known), `prompt_hash`, and timing metadata.
- **Duration computation**: Maintain an in-memory accumulator (daemon) or on-demand reducer that calculates time spent per state between transition events, rolling up into derived metrics (mean runtime, 95th percentile) per task type.
- **Task classification**: Hash the normalized prompt or tag commands (`leaf plan send --tags refactor,repo-a`) so predictions compare "similar" work. Store mapping in metrics DB.
- **Predictive estimates**: Use exponentially weighted moving averages seeded with bootstrap defaults (e.g., manual heuristics) until enough history accrues. Surfaced via `leaf status --with-estimate` or `leaf wait --explain`.
- **Anomaly detection**: Flag tasks exceeding `max(2×expected, expected + 2σ)` and emit `anomaly` events. Surface in CLI as warnings and optionally trigger notifications.
- **Smart wait integration**: `leaf wait` should display `elapsed`, `expected`, `p95`, and `ETA`. Allow `--alert` to notify when anomalies fire or completion occurs.
- **Data retention**: Rotate JSONL logs daily, compress archives, and keep aggregates in SQLite to retain historical baselines without ballooning disk usage.

## Responses to Open Questions
1. **State detection implementation**: Start with Go for the core detection binary. It offers fast startup, strong JSON support, dependency-free static binaries, and easier future expansion (metrics, daemon). Bash wrappers can remain for CLI glue while delegating heavy logic to Go.
2. **Cache TTL**: Use adaptive TTLs—default 1s, shorten to 250ms when a window is `working`, and extend to 5s when `ready/done`. Always bypass cache on explicit `--force`, send-keys, or when elapsed time since capture exceeds the expected task duration / 4.
3. **State granularity**: Keep the five canonical states but attach optional `substate` metadata (e.g., `working:running-tests`). This gives richer context without exploding the primary state machine.
4. **Notifications**: Support a pluggable notifier interface. Ship tmux status-line badges and macOS notifications out of the box; leave room for webhook/Slack integrations by emitting JSON events other tools can consume.
5. **Backwards compatibility**: Detect missing `.leaf` directories and fall back to ephemeral in-memory caching and plain stdout logs. Warn users once, auto-create structure on first run, and keep CLI behavior functional even without advanced logging.

## Edge Cases & Failure Modes
- Empty or newly created panes where capture returns nothing—must report `unknown` with low confidence rather than `ready`.
- High-velocity output (e.g., `tail -f` panes) can exhaust tmux capture limits; implement chunked reads and consider throttling watch frequency.
- Agent prompt changes (different models, locales) will break regex-based heuristics unless configs are versioned per agent/model.
- tmux session renames or detached sessions could make `leaf status` fail silently; commands should verify session existence first and suggest `leaf init`.
- Running multiple leaf sessions concurrently could interleave log streams; include `session_id` in every log/event entry to keep data distinct.
- Sandbox restrictions (no tmux access) or remote execution contexts must gracefully degrade, likely by returning an actionable error early.

## Implementation Priority Suggestions
1. Deliver core CLI replacements (`status`, `peek`, `wait`, `log`) backed by a robust detection binary and JSON event logging—this unblocks tmux blacklisting.
2. Add caching plus structured logs, then integrate smart waiting (elapsed + basic ETA) fed by historical durations.
3. Introduce anomaly detection and notification hooks once enough telemetry exists.
4. Iterate on daemon/service architecture for continuous monitoring, followed by predictive dashboards and richer analytics.

## Additional Concerns & Improvements
- Document clear contribution guidelines for adding new agent heuristics, including sample captures and regression tests.
- Provide a `leaf doctor` command to validate directory layout, permissions, and tmux connectivity before agents rely on the observability layer.
- Consider exposing an internal gRPC/HTTP API for Claude and other tooling to query state without invoking CLIs repeatedly.
- Plan for security/privacy: avoid logging sensitive prompts verbatim; allow redaction or hashing policies configurable per workspace.
- Ensure CLI remains fast by profiling `tmux capture-pane` usage under load; if necessary, shell out to `tmux display-message -F "#{pane_active}"` to short-circuit idle panes.

