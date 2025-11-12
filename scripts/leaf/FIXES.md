# Leaf Fixes - Session 2025-10-19

## Issues Fixed

### 1. Silent Failure Bug (LEAF_PROJECT_ROOT)
- **Problem**: `tmux show-environment PWD` returned empty, causing `mkdir -p /.leaf/` permission denied
- **Fix**: Created `resolve_project_dir()` helper (line 122) with fallback chain
- **Implementation**: Persisted `LEAF_PROJECT_ROOT` in tmux environment during `cmd_init`
- **Files modified**:
  - `leaf.sh:122` - new `resolve_project_dir()` function
  - `leaf.sh:220,242` - persist env var in `cmd_init`
  - `leaf.sh:408` - use helper in `cmd_list`
  - `leaf.sh:474` - use helper in `cmd_plan`
  - `leaf.sh:926` - use helper in `cmd_do_tasks`

### 2. Invalid Codex CLI Flags
- **Problem**: `codex exec` doesn't recognize `--workspace`, `--label`, `--log-level`, `--otel-debug`, `--sandbox`, `--timeout`
- **Fix**: Replaced with `-c` config overrides
- **Changes** (line 737-740):
  ```bash
  # REMOVED invalid flags:
  # CMD+=("--workspace" "$PROJECT_ROOT")
  # CMD+=("--label" "$RUN_LABEL")
  # CMD+=("--log-level" "debug")
  # CMD+=("--otel-debug")
  # CMD+=("--sandbox" "project")

  # ADDED config overrides:
  CMD+=("-c" "logging.level=debug")
  CMD+=("-c" "sandbox.default_mode=workspace-write")
  CMD+=("-c" "telemetry.otel.enabled=true")
  CMD+=("-c" "telemetry.otel.exporter=debug")
  ```

### 3. Missing Prompt Argument
- **Problem**: `codex exec` requires prompt via stdin or argument
- **Fix**: Pipe instructions file to codex exec via stdin (line 765-769)
- **Implementation**:
  ```bash
  if [[ -n "$INSTRUCTIONS_FILE" && -f "$INSTRUCTIONS_FILE" ]]; then
    "${CMD[@]}" < "$INSTRUCTIONS_FILE" || exit_code=$?
  else
    "${CMD[@]}" || exit_code=$?
  fi
  ```

### 4. Invalid YAML codex_args
- **Problem**: `--max-iterations` flag doesn't exist in new Codex CLI
- **Fix**: Removed `codex_args` line from `.leaf/plans/refactor-plan-to-repl.yml`
- **Note**: Config overrides should be done via main code, not YAML args

## Verification

✅ `leaf plan refactor-plan-to-repl --no-attach` successfully starts Codex exec
✅ Codex is working on the REPL refactoring task
✅ All tmux windows functioning correctly

## Known Issues for Next Session

1. **Window naming**: "codex-" vs "codex" inconsistency (cosmetic, will be fixed in next `leaf init`)
2. **Sandbox limitations**: `/bin/ps` operation not permitted (Codex sandbox restriction, not a bug)
3. **tmux send-keys pattern**: Need to implement double send-keys for REPL interaction (text, then C-m separately)

## Ready for Next Session

The following are ready to implement once Codex completes the REPL refactoring plan:

1. **Refactor `cmd_plan` for REPL mode**:
   - `leaf plan` → start/attach Codex REPL
   - `leaf plan send "<prompt>"` → send to REPL via tmux (using two send-keys commands)

2. **Create `cmd_do_codex`**:
   - Move current one-off exec logic from `cmd_plan`
   - Keep YAML task file support
   - Use `leaf-do-status` window pattern

3. **Update documentation**:
   - Help text for new commands
   - Migration notes for existing `.leaf/plans/*.yml` files

## How to Restart

```bash
# Kill current session
tmux kill-session -t leaf-leaf

# Start fresh session
cd ~/dotfiles/scripts/leaf
./leaf.sh init
./leaf.sh attach
```

All fixes are saved in `leaf.sh`. The session is ready for a clean restart.
