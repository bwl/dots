# Leaf Observability Implementation Plan

## Responsibility Split (Go vs Bash)
- Go (`internal/observer`): tmux capture abstraction, heuristic evaluation, state classification, JSON output, cache read/write, execution event logging.
- Go (`cmd/leaf-detect-state`): CLI binary that accepts flags/env, invokes observer package, prints JSON, sets exit codes.
- Bash (`leaf.sh`): argument parsing, user-facing commands (`status`, `peek`, `tail`, `wait`, `log`, `timeline`), invocation of Go binary, formatting output, retries on recoverable failures.
- Bash scripts (`scripts/leaf/lib/*.sh`): shared helpers for CLI commands, event log utilities (appending to JSONL), input validation.

## Directory & File Layout (relative to `scripts/leaf/`)
```
scripts/leaf/
├── leaf.sh
├── OBSERVABILITY-IMPLEMENTATION.md
├── LEAF-OBSERVABILITY-PLAN.md
├── go.mod
├── go.sum
├── cmd/
│   └── leaf-detect-state/
│       └── main.go
├── internal/
│   └── observer/
│       ├── capture.go
│       ├── classify.go
│       ├── config.go
│       ├── cache.go
│       ├── events.go
│       ├── model.go
│       ├── heuristics/
│       │   ├── codex.json
│       │   └── cliffy.json
│       └── observer_test.go
├── lib/
│   ├── detect.sh            # wraps Go binary
│   ├── output.sh            # formatting utilities
│   └── log.sh               # event append helpers
├── .leaf/
│   ├── logs/
│   │   └── executions.jsonl
│   ├── cache/
│   │   └── state-cache.json
│   └── heuristics/          # optional workspace overrides
│       ├── codex.json
│       └── cliffy.json
└── testdata/
    ├── captures/
    │   └── codex/
    │       ├── ready.txt
    │       ├── working.txt
    │       └── failed.txt
    └── events/
        └── sample.jsonl
```

## JSON Schemas

### State Detection Output (`leaf-detect-state`)
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "LeafStateDetection",
  "type": "object",
  "required": ["session", "window", "pane_id", "state", "confidence", "timestamp", "sample"],
  "properties": {
    "session": {"type": "string"},
    "window": {"type": "string"},
    "pane_id": {"type": "string"},
    "state": {"type": "string", "enum": ["ready", "working", "done", "waiting", "failed", "unknown"]},
    "substate": {"type": ["string", "null"]},
    "confidence": {"type": "number", "minimum": 0, "maximum": 1},
    "timestamp": {"type": "string", "format": "date-time"},
    "elapsed_since_last_state_seconds": {"type": ["number", "null"], "minimum": 0},
    "indicators": {
      "type": "array",
      "items": {"type": "object", "required": ["name", "score"], "properties": {
        "name": {"type": "string"},
        "score": {"type": "number"},
        "matches": {"type": "array", "items": {"type": "string"}}
      }}
    },
    "sample": {
      "type": "object",
      "required": ["lines", "line_count"],
      "properties": {
        "lines": {"type": "array", "items": {"type": "string"}},
        "line_count": {"type": "integer", "minimum": 0},
        "truncated": {"type": "boolean"}
      }
    },
    "cache": {
      "type": "object",
      "required": ["source"],
      "properties": {
        "source": {"type": "string", "enum": ["fresh", "cache"]},
        "valid_until": {"type": ["string", "null"], "format": "date-time"}
      }
    },
    "metrics": {
      "type": "object",
      "properties": {
        "expected_duration_seconds": {"type": ["number", "null"], "minimum": 0},
        "remaining_estimate_seconds": {"type": ["number", "null"], "minimum": 0}
      }
    }
  }
}
```

### Execution Event Log Entry (`.leaf/logs/executions.jsonl`)
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "LeafExecutionEvent",
  "type": "object",
  "required": ["id", "timestamp", "session", "window", "pane_id", "event", "state"],
  "properties": {
    "id": {"type": "string"},
    "timestamp": {"type": "string", "format": "date-time"},
    "session": {"type": "string"},
    "window": {"type": "string"},
    "pane_id": {"type": "string"},
    "event": {
      "type": "string",
      "enum": ["state.enter", "state.exit", "task.start", "task.finish", "command.wait.start", "command.wait.finish", "anomaly.detected"]
    },
    "state": {"type": "string"},
    "substate": {"type": ["string", "null"]},
    "confidence": {"type": ["number", "null"], "minimum": 0, "maximum": 1},
    "elapsed_seconds": {"type": ["number", "null"], "minimum": 0},
    "expected_seconds": {"type": ["number", "null"], "minimum": 0},
    "metadata": {
      "type": "object",
      "additionalProperties": {"type": ["string", "number", "boolean", "null"]}
    },
    "prompt_hash": {"type": ["string", "null"]},
    "task_tags": {
      "type": "array",
      "items": {"type": "string"}
    }
  }
}
```

### Cache File Entry (`.leaf/cache/state-cache.json`)
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "LeafStateCache",
  "type": "object",
  "required": ["version", "entries"],
  "properties": {
    "version": {"type": "integer"},
    "entries": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["session", "window", "pane_id", "state", "confidence", "captured_at", "valid_until", "hash"],
        "properties": {
          "session": {"type": "string"},
          "window": {"type": "string"},
          "pane_id": {"type": "string"},
          "state": {"type": "string"},
          "substate": {"type": ["string", "null"]},
          "confidence": {"type": "number", "minimum": 0, "maximum": 1},
          "captured_at": {"type": "string", "format": "date-time"},
          "valid_until": {"type": "string", "format": "date-time"},
          "hash": {"type": "string"},
          "sample_line_count": {"type": "integer", "minimum": 0}
        }
      }
    }
  }
}
```

### Heuristic Configuration (`internal/observer/heuristics/*.json`)
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "LeafHeuristicsConfig",
  "type": "object",
  "required": ["agent", "version", "states"],
  "properties": {
    "agent": {"type": "string"},
    "version": {"type": "string"},
    "default_state": {"type": "string"},
    "states": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "patterns"],
        "properties": {
          "name": {"type": "string"},
          "substates": {
            "type": "array",
            "items": {"type": "string"}
          },
          "threshold": {"type": "number"},
          "patterns": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["regex", "score"],
              "properties": {
                "regex": {"type": "string"},
                "score": {"type": "number"},
                "substate": {"type": ["string", "null"]},
                "indicators": {
                  "type": "array",
                  "items": {"type": "string"}
                }
              }
            }
          }
        }
      }
    },
    "fallback": {
      "type": "object",
      "required": ["state", "confidence"],
      "properties": {
        "state": {"type": "string"},
        "confidence": {"type": "number"}
      }
    }
  }
}
```

## Build Sequence (Dependency Ordered)
1. Initialize Go module (`go.mod`, `go.sum`) and directory scaffolding under `cmd/` and `internal/`.
2. Implement `capture.go` to wrap tmux calls (`capture-pane`, pane existence checks) with unit-testable interfaces (using command executor abstraction).
3. Implement configuration loader (`config.go`) that merges built-in heuristics with optional `.leaf/heuristics/*.json` overrides.
4. Implement heuristic evaluator (`classify.go`) that calculates scores, resolves state/substate, and produces indicator metadata.
5. Implement cache subsystem (`cache.go`) to read/write the cache JSON, enforce TTL, and invalidate on hash changes.
6. Implement metrics/event module (`events.go`) to append JSONL logs and compute elapsed expectations.
7. Wire modules together in `model.go` into a single `Observer` entry point returning the state detection schema.
8. Build CLI binary (`cmd/leaf-detect-state/main.go`) that parses CLI arguments, invokes observer, prints JSON, sets exit codes.
9. Create Bash wrapper `lib/detect.sh` that orchestrates binary invocation, handles retries, and surfaces errors.
10. Update `leaf.sh` to add new user commands (`status`, `peek`, `wait`, `log`, `timeline`) and integrate wrapper.
11. Add formatting helper scripts (`lib/output.sh`) and logging helper (`lib/log.sh`).
12. Add test fixtures under `testdata/` and write Go unit/integration tests plus Bash tests covering CLI flows.

## Parallel Work Streams
- Go backend team can progress through steps 2–8 sequentially.
- Bash CLI team can begin steps 9–11 once step 8 supplies a stable binary interface; they can draft command parsing in parallel using mocked outputs.
- Testing team can prepare fixtures (step 12) in parallel with step 4 to ensure heuristics are validated as soon as the evaluator exists.
- Documentation updates (`LEAF-OBSERVABILITY-PLAN.md`, usage docs) proceed in parallel after step 9.

## Integration Contract (`leaf.sh` ↔ `leaf-detect-state`)
- Invocation: `leaf-detect-state --session "$LEAF_SESSION" --window "$window" --pane "$pane_id" --lines 50 --cache-mode auto`.
- Input: No stdin. All parameters via flags. Environment variables used: `LEAF_CACHE_PATH`, `LEAF_HEURISTICS_DIR`, `LEAF_LOG_PATH`.
- Output: Single JSON object per invocation matching the State Detection schema. Stdout only.
- Exit codes:
  - `0`: Success with confident state (confidence ≥ 0.6).
  - `10`: Success with low confidence (< 0.6); CLI treats as `unknown`.
  - `20`: Pane or session not found; CLI displays friendly message.
  - `30`: tmux interaction failure; CLI retries once before surfacing error.
  - `40`: Configuration error (invalid heuristics JSON); CLI logs and falls back to built-in defaults.
  - `50`: Internal error; CLI surfaces failure and suggests `leaf doctor`.
- Cached responses set `"cache.source": "cache"`; fresh captures set `"cache.source": "fresh"`.

## Testing Strategy
- Go modules: Unit tests for `capture`, `classify`, `config`, `cache`, and `events` using fixtures. Integration tests using recorded captures to assert state outputs and confidence scores.
- Go binary: CLI tests using `go test` with `exec.Command` to ensure flags, exit codes, and JSON output conform to schema.
- Bash wrapper and commands: `bats` tests that mock `leaf-detect-state` binary via environment override to ensure CLI handles low confidence, missing panes, cache hits, and formatting.
- Schema validation: CI step that runs `go run cmd/leaf-detect-state` against `testdata/captures/*` and validates JSON against schemas using `cue` or `gojsonschema`.
- Cache and logging: Tests that write to temporary directories, assert JSON structures, and confirm TTL behavior.
- End-to-end: Manual script that spins up a tmux session with controlled output, runs `leaf status`, `leaf wait`, and verifies state transitions and event logs.
