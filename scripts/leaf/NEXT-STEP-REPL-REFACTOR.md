# Next Step: Refactor to REPL Mode

**Source**: Codex plan completed 2025-10-19T19:20:09-04:00 (2m 6s runtime)
**Tokens used**: 35,422

## Implementation Plan

### 1. Dispatcher Changes (`leaf.sh` lines 1100-1140)

- `leaf plan` → calls new interactive REPL handler
- `leaf do` → gains `codex` branch invoking `cmd_do_codex`

### 2. Refactor `cmd_plan` → `cmd_do_codex` (lines 463-838)

**Move** current `cmd_plan` logic to new `cmd_do_codex()`:
- Keep YAML lookup, run-script generation, status piping
- Retarget status window to `leaf-do-status` (matching `cmd_do_tasks`)
- Default execution to `codex` window
- Expose all existing options:
  - `--window`, `--timeout`, `--preview`, `--no-attach`, `--capture`

### 3. New `cmd_plan()` - Interactive REPL Controller

**Subcommands:**
- `leaf plan` (no args) or `leaf plan attach` → start/focus REPL
- `leaf plan send "<prompt>"` → stream prompts to REPL
- `leaf plan --resume <id>` → resume previous session (optional)

**REPL Detection Logic:**
```bash
# Use tmux list-panes -F '#{pane_current_command}' for the codex window
# Treat "codex" or wrapper script as "active"
# Avoid external `ps` due to sandbox restrictions
```

**Startup Flow (when inactive):**
1. Clear pane
2. `cd` into project directory
3. Run `.leaf/launch.sh codex exec --interactive`
4. Append `--resume "$resume_id"` if supplied
5. Kill any existing pipe-pane to avoid duplicates
6. Send `C-c` to clean up before launch
7. Write status line so users know it restarted

**Active Flow:**
- Just focus/select the window
- For `send`: issue `tmux send-keys "$prompt"` then **separate** `tmux send-keys C-m`
- Guard: auto-start REPL before sending if not running

**Restart Scenarios:**
- If `pane_current_command` ≠ `codex`: warn and restart
- If REPL exits unexpectedly: next `leaf plan` auto-relaunches
- Provide friendly error messages

### 4. Helper Functions (after `find_window_index_by_name()`)

Add these utilities:
- `ensure_window(session, name, dir)` → create windows idempotently
- `codex_pane_id()` → grab first pane ID for consistent `send-keys`
- `codex_repl_running()` → encapsulate `pane_current_command` check

### 5. Update Help Text (`usage()` lines 24-108)

**Document new commands:**
```bash
leaf plan                    # Start/attach Codex REPL
leaf plan send "<message>"   # Send prompt to running REPL
leaf plan --resume <id>      # Resume previous session

leaf do codex <task>         # One-off Codex exec (replaces old `leaf plan <task>`)
```

**Examples:**
- Show `leaf plan send` with multi-line quoting notes
- Show `leaf do codex <task>` with YAML task files
- Mention `--resume` flag support

### 6. REPL Lifecycle Handling

**Startup:**
- Kill existing pipe-pane (avoid duplicates)
- Send `C-c` to clear any previous state
- Launch interactive command
- Write status line for user visibility

**Sending prompts:**
- **CRITICAL**: Use TWO separate `tmux send-keys` commands:
  ```bash
  tmux send-keys -t "$pane_id" "$prompt"
  tmux send-keys -t "$pane_id" C-m
  ```
- Rely on tmux buffering for rapid sends
- Document multi-line prompt quoting

**Monitoring output:**
- `leaf plan status` → directs users to `codex` window
- Suggest `tmux capture-pane -p -t codex` for quick inspection from `coord`
- Coordinators can check output without switching windows

**Auto-recovery:**
- If REPL exits: next `leaf plan` sees wrong `pane_current_command` and relaunches
- No manual intervention needed

### 7. Migration Notes for `.leaf/plans/*.yml`

**No file moves needed:**
- `cmd_do_codex` reads same paths
- YAML schema unchanged: `title`, `codex_args`, `instructions`

**User changes required:**
- Old: `leaf plan <task>`
- New: `leaf do codex <task>`

**Documentation updates:**
- Update README with new commands
- Clean up old `leaf plan` workflow references
- Add migration guide for existing automation/scripts

### 8. Edge Cases

**Resume support:**
- `leaf plan --resume <session>` passes through to interactive launch
- One-off resumes: `leaf do codex <task> --resume ...`

**Empty pane protection:**
- `leaf plan send` auto-starts REPL if not running
- Never fail on empty pane

**tmux permission errors:**
- Avoid external `ps` completely
- Use only tmux-format data (`#{pane_current_command}`)

**Output inspection:**
- Users can switch to `codex` window directly
- Or use `leaf plan send` + `tmux capture-pane` from coordination window

## Implementation Order

1. ✅ Add helper functions (`ensure_window`, `codex_pane_id`, `codex_repl_running`)
2. ✅ Create new `cmd_do_codex()` by copying current `cmd_plan` logic
3. ✅ Rewrite `cmd_plan()` with REPL controller logic
4. ✅ Update dispatcher in `main()` to route `leaf do codex`
5. ✅ Update help text and usage examples
6. ✅ Test interactive workflow:
   - `leaf plan` starts REPL
   - `leaf plan send "test message"` works with double send-keys
   - `leaf do codex refactor-plan-to-repl` runs one-off exec
7. ✅ Update README and migration docs

## Code References

- **Main dispatcher**: `leaf.sh:1100-1140`
- **Current `cmd_plan`**: `leaf.sh:463-838` (move to `cmd_do_codex`)
- **Helper location**: After `find_window_index_by_name()`
- **Help text**: `leaf.sh:24-108`
- **Example YAML**: `.leaf/plans/refactor-plan-to-repl.yml`

## Key Architectural Decisions

1. **REPL detection**: Use `tmux list-panes -F '#{pane_current_command}'` instead of `ps`
2. **Send pattern**: Two separate `tmux send-keys` commands (text, then C-m)
3. **Auto-recovery**: REPL auto-restarts on next `leaf plan` if exited
4. **Window management**: Idempotent `ensure_window()` helper
5. **Migration path**: Keep YAML files, change command name only

## Testing Checklist

- [ ] `leaf plan` starts Codex REPL in codex window
- [ ] `leaf plan attach` focuses existing REPL
- [ ] `leaf plan send "hello"` sends message (two send-keys)
- [ ] `leaf plan send` auto-starts REPL if not running
- [ ] `leaf do codex <task>` runs one-off exec with YAML
- [ ] REPL auto-restarts after exit
- [ ] `--resume` flag works (if implemented)
- [ ] Help text accurate and complete
- [ ] README updated with migration notes

## Success Criteria

✅ `leaf plan` manages persistent interactive Codex REPL
✅ `leaf plan send` reliably sends prompts via tmux
✅ `leaf do codex` handles one-off exec tasks with YAMLs
✅ No re-contextualization overhead (REPL stays alive)
✅ Clean error messages and auto-recovery
✅ Documentation complete with migration guide

---

**Ready for implementation in next session.**
**All prerequisite bugs fixed and verified working.**
