# tmux Command Reference for Coding Agents

This document describes the tmux command-line interface for programmatic interaction with tmux sessions, windows, and panes. Use these commands to send instructions to running processes and retrieve their output.

## Target Syntax

tmux uses a hierarchical addressing system: `session:window.pane`

- **Session**: Named or numbered (e.g., `home-base`, `$SESSION_NAME`)
- **Window**: Index (0-based) or name (e.g., `:0`, `:claude`)
- **Pane**: Index within window (0-based, e.g., `.0`, `.1`)

### Examples

```bash
# Target pane 0 in window 0 of session "home-base"
home-base:0.0

# Target pane 0 in window named "claude" of session "home-base"
home-base:claude.0

# Target current pane (relative addressing)
.

# Target current window
:
```

## Sending Commands to Panes

### `tmux send-keys`

Send keystrokes to a target pane as if typed by a user.

```bash
# Send a command and press Enter (C-m = Ctrl+m = Enter)
tmux send-keys -t "session:window.pane" "command here" C-m

# Send a command without pressing Enter
tmux send-keys -t "session:window.pane" "command here"

# Send literal strings (prevents interpretation)
tmux send-keys -t "session:window.pane" -l "literal text"

# Send special keys
tmux send-keys -t "session:window.pane" C-c    # Ctrl+C (interrupt)
tmux send-keys -t "session:window.pane" C-d    # Ctrl+D (EOF)
tmux send-keys -t "session:window.pane" Up     # Up arrow
tmux send-keys -t "session:window.pane" Enter  # Enter key
```

### Key Notation

- `C-m` or `Enter` - Enter/Return key
- `C-c` - Ctrl+C (interrupt signal)
- `C-d` - Ctrl+D (EOF)
- `C-z` - Ctrl+Z (suspend)
- `Up`, `Down`, `Left`, `Right` - Arrow keys
- `Space`, `Tab`, `BSpace` - Special keys

### Practical Examples

```bash
# Run a shell command
tmux send-keys -t "home-base:tasks.0" "npm test" C-m

# Execute Python code
tmux send-keys -t "home-base:1.0" "python -c 'print(2+2)'" C-m

# Send multi-line input (heredoc pattern)
tmux send-keys -t "home-base:0.0" "cat << 'EOF'" C-m
tmux send-keys -t "home-base:0.0" "line 1" C-m
tmux send-keys -t "home-base:0.0" "line 2" C-m
tmux send-keys -t "home-base:0.0" "EOF" C-m

# Interrupt a running process
tmux send-keys -t "home-base:2.0" C-c

# Re-run the last command
tmux send-keys -t "home-base:0.0" Up C-m
```

## Reading Output from Panes

### `tmux capture-pane`

Capture the visible content or scrollback buffer of a pane.

```bash
# Capture visible content and print to stdout
tmux capture-pane -t "session:window.pane" -p

# Capture with ANSI escape sequences preserved
tmux capture-pane -t "session:window.pane" -p -e

# Capture entire scrollback history
tmux capture-pane -t "session:window.pane" -p -S -

# Capture specific line range (e.g., last 100 lines)
tmux capture-pane -t "session:window.pane" -p -S -100

# Capture and save to a file
tmux capture-pane -t "session:window.pane" -p > output.txt

# Join wrapped lines
tmux capture-pane -t "session:window.pane" -p -J
```

### Options

- `-p` - Print to stdout (required for shell capture)
- `-e` - Include ANSI escape codes (colors, formatting)
- `-J` - Join wrapped lines
- `-S start` - Start line (negative = from end, `-` = beginning)
- `-E end` - End line (negative = from end)

### Practical Examples

```bash
# Get last command output (last 50 lines)
tmux capture-pane -t "home-base:0.0" -p -S -50

# Monitor a long-running process
tmux capture-pane -t "home-base:tasks.0" -p -S -20

# Capture only new output (save cursor position between reads)
# Read last N lines where N is determined by your tracking
tmux capture-pane -t "home-base:1.0" -p -S -10

# Search output for a pattern
tmux capture-pane -t "home-base:0.0" -p | grep "ERROR"

# Check if a command finished
tmux capture-pane -t "home-base:2.0" -p -S -5 | tail -1
```

## Session and Window Management

### Checking Existence

```bash
# Check if a session exists
tmux has-session -t "session-name" 2>/dev/null
# Exit code 0 = exists, 1 = does not exist

# List all sessions
tmux list-sessions
tmux ls

# List windows in a session
tmux list-windows -t "session-name"

# List panes in a window
tmux list-panes -t "session:window"
```

### Getting Information

```bash
# Get current session name
tmux display-message -p '#S'

# Get current window index
tmux display-message -p '#I'

# Get current window name
tmux display-message -p '#W'

# Get current pane index
tmux display-message -p '#P'

# Get pane dimensions
tmux display-message -p -t "session:window.pane" '#{pane_width}x#{pane_height}'

# Get full target path
tmux display-message -p -t "session:window.pane" '#{session_name}:#{window_index}.#{pane_index}'

# Check if pane is running a command
tmux display-message -p -t "session:window.pane" '#{pane_current_command}'

# Get pane working directory
tmux display-message -p -t "session:window.pane" '#{pane_current_path}'
```

### Format Variables

Useful format variables for `display-message -p`:

- `#{session_name}` - Session name
- `#{window_index}` - Window index
- `#{window_name}` - Window name
- `#{pane_index}` - Pane index
- `#{pane_current_command}` - Currently running command
- `#{pane_current_path}` - Working directory
- `#{pane_width}` - Pane width in characters
- `#{pane_height}` - Pane height in lines
- `#{pane_pid}` - Process ID
- `#{pane_dead}` - 1 if pane is dead, 0 otherwise

## Workflow Patterns for Agents

### Pattern 1: Execute and Wait

```bash
# Send command
tmux send-keys -t "home-base:0.0" "long-running-task" C-m

# Poll for completion (check for shell prompt or specific output)
while true; do
  OUTPUT=$(tmux capture-pane -t "home-base:0.0" -p -S -1)
  echo "$OUTPUT" | grep -q "task completed" && break
  sleep 1
done

# Capture final output
tmux capture-pane -t "home-base:0.0" -p -S -50
```

### Pattern 2: Send and Read

```bash
# Send a command that produces immediate output
tmux send-keys -t "home-base:1.0" "date" C-m

# Wait briefly for execution
sleep 0.5

# Read result
tmux capture-pane -t "home-base:1.0" -p -S -5
```

### Pattern 3: Interactive Session

```bash
# Start interactive program
tmux send-keys -t "home-base:2.0" "python" C-m
sleep 0.2

# Send input
tmux send-keys -t "home-base:2.0" "x = 42" C-m
sleep 0.1

tmux send-keys -t "home-base:2.0" "print(x * 2)" C-m
sleep 0.1

# Read output
tmux capture-pane -t "home-base:2.0" -p -S -10

# Exit interactive session
tmux send-keys -t "home-base:2.0" "exit()" C-m
```

### Pattern 4: Monitoring Background Process

```bash
# Start background process
tmux send-keys -t "home-base:tasks.0" "npm run dev" C-m

# Check periodically
tmux capture-pane -t "home-base:tasks.0" -p -S -20 | tail -5

# Look for specific startup message
while ! tmux capture-pane -t "home-base:tasks.0" -p | grep -q "Server started"; do
  sleep 1
done
```

### Pattern 5: Error Detection

```bash
# Execute command
tmux send-keys -t "home-base:0.0" "risky-command" C-m
sleep 1

# Check for errors in output
OUTPUT=$(tmux capture-pane -t "home-base:0.0" -p -S -50)
if echo "$OUTPUT" | grep -qE "(ERROR|FAIL|Exception)"; then
  echo "Command failed"
  echo "$OUTPUT"
  exit 1
fi
```

## Best Practices for Agents

### 1. Always Validate Targets

```bash
# Check session exists before sending commands
if ! tmux has-session -t "home-base" 2>/dev/null; then
  echo "Session 'home-base' does not exist"
  exit 1
fi
```

### 2. Use Appropriate Delays

After `send-keys`, allow time for command execution before reading output:
- Simple commands: 100-500ms
- Interactive programs: 200-1000ms
- Long-running tasks: poll periodically

### 3. Capture Sufficient Context

```bash
# Too little context (may miss output)
tmux capture-pane -t "target" -p -S -5

# Better (enough context for most commands)
tmux capture-pane -t "target" -p -S -50

# Full scrollback (for searching)
tmux capture-pane -t "target" -p -S -
```

### 4. Handle Escape Sequences

When capturing panes that contain colors or formatting:

```bash
# Strip ANSI codes for plain text processing
tmux capture-pane -t "target" -p | sed 's/\x1b\[[0-9;]*m//g'

# Or use -e to preserve them
tmux capture-pane -t "target" -p -e
```

### 5. Quote Targets and Commands

Always quote targets and commands to handle spaces and special characters:

```bash
tmux send-keys -t "session:window.pane" "command with spaces" C-m
```

### 6. Check Command Completion

Don't assume commands finish instantly:

```bash
# Bad: read immediately
tmux send-keys -t "target" "slow-command" C-m
tmux capture-pane -t "target" -p  # May capture incomplete output

# Good: wait or poll
tmux send-keys -t "target" "slow-command" C-m
sleep 2  # Or poll until done
tmux capture-pane -t "target" -p
```

### 7. Clean Up on Errors

```bash
# If you start a process, be prepared to interrupt it
tmux send-keys -t "target" "long-task" C-m

# On error or timeout:
tmux send-keys -t "target" C-c  # Interrupt
sleep 0.5
tmux send-keys -t "target" C-c  # Second interrupt if needed
```

## Advanced Techniques

### Piping Through tmux

```bash
# Send command output directly to a pane
echo "some data" | tmux load-buffer - && tmux paste-buffer -t "target"

# Alternative: use send-keys with heredoc
DATA=$(cat large-file.txt)
tmux send-keys -t "target" "$DATA" C-m
```

### Conditional Execution

```bash
# Execute command only if pane is idle
CURRENT_CMD=$(tmux display-message -p -t "target" '#{pane_current_command}')
if [ "$CURRENT_CMD" = "bash" ] || [ "$CURRENT_CMD" = "zsh" ]; then
  tmux send-keys -t "target" "my-command" C-m
else
  echo "Pane is busy running: $CURRENT_CMD"
fi
```

### Multi-Pane Coordination

```bash
# Send command to multiple panes
for PANE in 0.0 1.0 2.0; do
  tmux send-keys -t "home-base:$PANE" "echo 'sync'" C-m
done

# Wait for all to complete
for PANE in 0.0 1.0 2.0; do
  while ! tmux capture-pane -t "home-base:$PANE" -p | grep -q "sync"; do
    sleep 0.5
  done
done
```

## Common Gotchas

1. **Timing Issues**: Commands may not complete before you read output. Always add delays or poll.

2. **Scrollback Limits**: tmux has a scrollback limit (default 2000 lines). Old output is discarded.

3. **Pane Size**: Long lines wrap based on pane width. Use `-J` to join wrapped lines.

4. **Interactive Prompts**: Commands with prompts (y/n) require special handling:
   ```bash
   tmux send-keys -t "target" "dangerous-command" C-m
   sleep 0.5
   tmux send-keys -t "target" "y" C-m  # Answer prompt
   ```

5. **Shell Interpretation**: `send-keys` sends literal keystrokes. Shell quoting rules apply:
   ```bash
   # Wrong: shell interprets $VAR before sending
   tmux send-keys -t "target" "echo $VAR" C-m

   # Right: escape or use single quotes
   tmux send-keys -t "target" "echo \$VAR" C-m
   tmux send-keys -t "target" 'echo $VAR' C-m
   ```

6. **Command Not Found**: If a command doesn't exist, you'll see an error in the pane output. Always check capture results.

## Reference: Complete Command List

```bash
# Session management
tmux has-session -t <session>
tmux list-sessions
tmux new-session -d -s <session>
tmux kill-session -t <session>

# Window management
tmux list-windows -t <session>
tmux new-window -t <session> -n <name>
tmux kill-window -t <session:window>

# Pane management
tmux list-panes -t <session:window>
tmux split-window -t <session:window>
tmux kill-pane -t <session:window.pane>

# Sending input
tmux send-keys -t <target> <keys...>
tmux send-keys -t <target> -l <literal-text>

# Reading output
tmux capture-pane -t <target> -p
tmux capture-pane -t <target> -p -S <start> -E <end>

# Getting information
tmux display-message -p -t <target> '<format>'
tmux show-options -g
tmux show-window-options -t <target>
```

## Summary

For most agent tasks, you'll primarily use:

1. **`send-keys`** - Execute commands in panes
2. **`capture-pane`** - Read command output
3. **`display-message`** - Query pane/window/session state
4. **`has-session`** - Verify targets exist

Combine these with shell scripting, delays, and polling to create reliable automation workflows.
