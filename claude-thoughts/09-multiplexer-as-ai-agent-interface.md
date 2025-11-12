# Terminal Multiplexer as AI Agent Interface

## The Paradigm Shift

You asked me to think about the ideal workflow **for me as a coding agent**, not just for you as a human.

This completely reframes what a terminal multiplexer is for.

**Traditional view:**
Terminal multiplexer = tool for humans to manage multiple terminal sessions

**AI-era view:**
Terminal multiplexer = **state management and process isolation layer for AI coding agents**

## The Context Window Problem

### My Current Limitation

I have a 200k token context window. That sounds like a lot, but:

```
System prompt + tools: ~20k tokens
Your CLAUDE.md files: ~3k tokens
This conversation so far: ~70k tokens
───────────────────────────────────
Available for work: ~107k tokens
```

When I run a command that produces 10k lines of output, that's ~30k tokens. Gone. One `cargo build` and I've lost 30% of my working memory.

### The Inline Execution Problem

**Current workflow:**
1. You: "Build the project and fix any errors"
2. Me: Runs `cargo build` via Bash tool
3. Output: 10,000 lines of compiler output
4. All goes into my context
5. I analyze the errors
6. Fix code
7. Repeat
8. After 3-4 iterations: Context full, must compact

**Result**: I'm spending context on PROCESS OUTPUT, not CODE UNDERSTANDING.

### The Multiplexer Solution

**Alternative workflow:**
1. You: "Build the project and fix any errors"
2. Me: Starts build in pane 2: `tmux send-keys -t session:2 "cargo build" C-m`
3. Build runs in background
4. I continue working in pane 0
5. Periodically check: `tmux capture-pane -t session:2 -p | tail -20`
6. Only read last 20 lines (is it done? any errors?)
7. If errors, read more context: `tmux capture-pane -t session:2 -p -S -100`
8. Fix code
9. Restart build in pane 2
10. Check again

**Result**: I spend context on ACTUAL ERRORS, not 9,000 lines of "Compiling foo v1.2.3"

## The Key Insight: Selective Attention

Humans don't read every line of build output. They:
1. Start the build
2. Do something else
3. Check if it's done
4. If error, look at the relevant part

AI agents should work the same way.

**Separate execution context from attention context.**

Just because a process is running doesn't mean I need to see all its output. I can **sample** from it when needed.

## Ideal Workspace Architecture

### The Mission Control Layout

```
Project Session: "my-app"

┌─────────────────────────────────────────────────────────┐
│ Tab 0: CONTROL (main workspace)                         │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Claude Code interactive session                     │ │
│ │ - Full tool access (read, write, edit, bash)       │ │
│ │ - Can send commands to other tabs                  │ │
│ │ - Can selectively read from other tabs             │ │
│ │ - Primary focus: CODE and ARCHITECTURE             │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Tab 1: BUILD (compilation)                              │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ $ cargo watch -x build                              │ │
│ │ [Running... Compiling 234 crates...]                │ │
│ │                                                      │ │
│ │ Claude reads this ONLY when:                        │ │
│ │ - Checking if build is complete                     │ │
│ │ - Errors detected (last 50 lines)                   │ │
│ │ - User asks "did it build?"                         │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Tab 2: TEST (test runner)                               │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ $ cargo watch -x test                               │ │
│ │ running 47 tests                                    │ │
│ │ test result: ok. 45 passed; 2 failed                │ │
│ │                                                      │ │
│ │ Claude reads this:                                  │ │
│ │ - After making changes (did tests pass?)            │ │
│ │ - When failures occur (which tests? why?)           │ │
│ │ - Summary only (not full output)                    │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Tab 3: SERVE (development server)                       │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ $ npm run dev                                       │ │
│ │ Server running on http://localhost:3000             │ │
│ │ [Request logs streaming...]                         │ │
│ │                                                      │ │
│ │ Claude reads this:                                  │ │
│ │ - When debugging HTTP errors                        │ │
│ │ - When verifying endpoints work                     │ │
│ │ - Last 20 lines only                                │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Tab 4: SERVICES (database, redis, etc.)                 │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ $ docker-compose up                                 │ │
│ │ postgres_1  | ready to accept connections           │ │
│ │ redis_1     | Ready to accept connections           │ │
│ │                                                      │ │
│ │ Claude reads this:                                  │ │
│ │ - When verifying services started                   │ │
│ │ - When debugging connection errors                  │ │
│ │ - Rarely (only on issues)                           │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Floating Pane: TASKS (toggled with Ctrl-T)              │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Todo List (taskbook)                                │ │
│ │ [x] Set up development environment                  │ │
│ │ [>] Implement user authentication                   │ │
│ │ [ ] Add API rate limiting                           │ │
│ │ [ ] Write integration tests                         │ │
│ │                                                      │ │
│ │ Updated by Claude as work progresses                │ │
│ │ User can toggle visibility to check progress        │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Why This Is Powerful

**Tab 0 (Control)**:
- This is where I "live"
- My context is focused on CODE, not output spam
- I can read/write files, plan architecture, make decisions
- Clean workspace, no log clutter

**Tabs 1-4 (Background processes)**:
- Running continuously
- I **don't** keep their output in my context
- I **do** check them on-demand
- Like peripheral vision - I glance at them when needed

**The cost difference:**

Traditional approach:
```
Run cargo build
Output: 30k tokens
Fix error
Run cargo build again
Output: 30k tokens
...
After 4 iterations: 120k tokens on build output
```

Multiplexer approach:
```
Start cargo watch in tab 1
Check tab 1: "tmux capture-pane -t 1 -p | tail -10"
Output: 200 tokens (just the summary)
Fix error
Check tab 1 again: 200 tokens
...
After 4 iterations: 800 tokens total
```

**150x more efficient.**

## The Monitoring Pattern

### Instead of Blocking

Traditional:
```bash
# I run this
npm run build

# I wait...
# 10 seconds pass...
# Output comes back
# 5,000 lines
# All goes into context
```

### Async Monitoring

Multiplexer:
```bash
# Start in background (tab 1)
tmux send-keys -t session:1 "npm run build" C-m

# Continue work in tab 0
# (edit files, plan next steps)

# Check after 10 seconds
tmux capture-pane -t session:1 -p | tail -1
# Output: "Build completed in 8.3s"
# Cost: 10 tokens

# If error instead:
tmux capture-pane -t session:1 -p -S -30
# Last 30 lines (enough to see the error)
# Cost: 500 tokens instead of 5,000
```

### Event-Driven Instead of Polling

Even better:
```bash
# Start build with notification
tmux send-keys -t session:1 "npm run build && echo BUILD_COMPLETE || echo BUILD_FAILED" C-m

# Periodically check for sentinel
tmux capture-pane -t session:1 -p | grep -E "BUILD_COMPLETE|BUILD_FAILED"

# Only read full output if BUILD_FAILED
```

This is like async/await for AI agents.

## The Multi-Agent Pattern

### Your Current Fresh Setup Already Does This

```
Tab 0: Claude Code (general purpose)
Tab 1: Codex (codebase exploration specialist)
Tab 2: Cliffy (batch LLM tasks)
Tab 3: Lazygit (version control)
```

This is **distributed AI agents with role specialization**.

### The Coordination Opportunity

Current: These are separate tools, but they could **coordinate**.

**Example: Test-Driven Development Workflow**

User (in tab 0 to main Claude): "Implement user login feature with TDD"

Main Claude:
1. Writes failing test
2. Sends to test agent: `tmux send-keys -t session:2 "run tests" C-m`
3. Test agent reports failure
4. Main Claude writes implementation
5. Test agent confirms pass
6. Main Claude refactors
7. Test agent confirms still passing

### How This Would Work

**Setup:**
```bash
# Tab 0: Main Claude Code (you're talking to this one)
claude

# Tab 2: Test watcher Claude
# Hypothetical: another claude instance with specific instructions
claude --mode test-watcher --watch "npm test"
```

**Interaction:**
```bash
# Main Claude sends work to test Claude
tmux send-keys -t session:2 "Check: did my last change break tests?" C-m

# Main Claude reads response
tmux capture-pane -t session:2 -p -S -20
# "3 tests failing: auth.test.js:45, auth.test.js:67, user.test.js:12"

# Main Claude fixes
# Checks again
tmux capture-pane -t session:2 -p -S -5
# "All tests passing"
```

### The Multi-Agent Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    USER (you)                            │
│                       ↓                                  │
│              Tab 0: ARCHITECT CLAUDE                     │
│              (main coordinator)                          │
│                       ↓                                  │
│    ┌──────────────┬──┴───────────┬─────────────┐       │
│    ↓              ↓              ↓             ↓        │
│  Tab 1:        Tab 2:        Tab 3:       Tab 4:        │
│  BUILD         TEST           CODE         DOC          │
│  AGENT         AGENT          REVIEW       AGENT        │
│                                AGENT                     │
│                                                          │
│  Each agent:                                             │
│  - Specialized role                                      │
│  - Isolated context                                      │
│  - Reports to architect                                  │
│  - Can be "queried" on demand                            │
└──────────────────────────────────────────────────────────┘
```

**Architect Claude** (Tab 0):
- Talks to user
- Makes high-level decisions
- Delegates to specialist agents
- Synthesizes results
- Has full codebase context

**Build Agent** (Tab 1):
- Monitors build process
- Reports errors
- Suggests fixes
- Only knows about builds

**Test Agent** (Tab 2):
- Runs tests continuously
- Reports failures immediately
- Suggests test improvements
- Only knows about tests

**Code Review Agent** (Tab 3):
- Analyzes changes
- Checks for issues
- Suggests improvements
- Only knows about code quality

**Documentation Agent** (Tab 4):
- Updates docs as code changes
- Ensures docs stay in sync
- Only knows about documentation

### Why This Is Powerful

1. **Parallel work**: Multiple agents can work simultaneously
2. **Specialization**: Each agent focused on one thing
3. **Context isolation**: Each agent has minimal, relevant context
4. **Cost efficiency**: Don't need full context in every agent
5. **Async operation**: Long-running agents work in background

### Current Limitations

**This isn't fully supported yet**, but the primitives exist:

✅ I can send commands to different panes
✅ I can read from different panes
✅ I can parse structured output
✅ I can coordinate across panes

❌ No native "spawn another Claude instance in pane 2"
❌ No inter-agent communication protocol
❌ No shared knowledge base between instances

**But it's buildable.**

## Concrete Workflows

### Workflow 1: Web Development

**Setup:**
```bash
# Tab 0: Main work (Claude)
# Tab 1: Backend server (FastAPI/Express)
# Tab 2: Frontend dev server (Vite/Next)
# Tab 3: Test runner (Vitest/Jest)
# Tab 4: Database (Docker)
```

**User:** "Add a new API endpoint for user profiles"

**Claude workflow:**
1. **Tab 0**: Edit backend route file
2. **Check Tab 1**: `tmux capture-pane -t 1 -p | tail -5` (server reloaded?)
3. **Tab 0**: Edit frontend component
4. **Check Tab 2**: `tmux capture-pane -t 2 -p | tail -5` (compiled?)
5. **Tab 0**: Write test
6. **Check Tab 3**: `tmux capture-pane -t 3 -p | grep "test.*profile"` (test running?)
7. **Report**: "Done. Server running, tests passing."

**Context used**: ~2k tokens (just the code + 5-line checks)
**Context saved**: ~20k tokens (full server logs, build output, test output)

### Workflow 2: System Programming (Rust)

**Setup:**
```bash
# Tab 0: Main work (Claude)
# Tab 1: cargo watch -x build
# Tab 2: cargo watch -x test
# Tab 3: Running binary (./target/debug/app)
```

**User:** "Optimize the parser performance"

**Claude workflow:**
1. **Tab 0**: Read parser code, plan optimization
2. **Tab 0**: Edit parser.rs
3. **Check Tab 1**: `tmux capture-pane -t 1 -p | tail -3`
   - Output: "Finished dev [unoptimized] in 2.1s"
4. **Restart Tab 3**: `tmux send-keys -t 3 C-c "./target/debug/app test.txt" C-m`
5. **Read Tab 3**: `tmux capture-pane -t 3 -p`
   - Output: "Parsed in 142ms (was 380ms)"
6. **Check Tab 2**: `tmux capture-pane -t 2 -p | grep "test result"`
   - Output: "test result: ok. 24 passed"
7. **Report**: "Parser is now 2.7x faster. All tests passing."

**Context used**: ~3k tokens
**Context saved**: ~40k tokens (full build logs, test output)

### Workflow 3: Complex Refactoring

**Setup:**
```bash
# Tab 0: Main work (Claude)
# Tab 1: Full test suite (runs once when triggered)
# Tab 2: TypeScript compiler (tsc --watch)
# Tab 3: Linter (eslint --watch)
```

**User:** "Refactor the authentication system to use dependency injection"

**Claude workflow:**
1. **Tab 0**: Analyze current code, plan refactoring
2. **Tab 0**: Create new interfaces
3. **Check Tab 2**: `tmux capture-pane -t 2 -p | tail -5` (type errors?)
4. **Tab 0**: Refactor AuthService
5. **Check Tab 2**: Still type errors?
6. **Tab 0**: Update all usages
7. **Check Tab 2**: Types good?
8. **Check Tab 3**: `tmux capture-pane -t 3 -p | grep "error"` (lint errors?)
9. **Trigger Tab 1**: `tmux send-keys -t 1 "npm test" C-m`
10. **Wait 30 seconds** (do other work in Tab 0)
11. **Check Tab 1**: `tmux capture-pane -t 1 -p | tail -10`
    - Output: "Tests: 127 passed, 127 total"
12. **Report**: "Refactoring complete. All types check, no lint errors, all tests pass."

**Context used**: ~5k tokens (code + incremental checks)
**Context saved**: ~60k tokens (full test output, compiler output)

## State Persistence

### The Hidden Benefit

Panes maintain state across my interactions:
- Working directory
- Environment variables
- Running processes
- Shell history

This means I can **set up once, reuse many times**.

**Example:**

```bash
# Initial setup in Tab 3 (Python project)
tmux send-keys -t session:3 "cd backend" C-m
tmux send-keys -t session:3 "source venv/bin/activate" C-m
tmux send-keys -t session:3 "export DJANGO_SETTINGS_MODULE=config.settings.dev" C-m
tmux send-keys -t session:3 "python manage.py runserver" C-m

# Now Tab 3 has:
# - Correct directory
# - Virtual env activated
# - Env vars set
# - Server running

# Later, I can send commands to this pane
# Without re-establishing state
tmux send-keys -t session:3 C-c "python manage.py migrate" C-m

# The virtualenv, directory, env vars are still there
```

**Without multiplexer**, every Bash command I run starts fresh:
- No working directory
- No environment
- No state

**With multiplexer**, I build up stateful environments and reuse them.

This is like **dependency injection for execution contexts**.

## The Vision: Fresh as AI Mission Control

Your `fresh` script is already 80% of the way there.

**Current Fresh:**
- Tab 0: claude (me)
- Tab 1: codex (specialized agent)
- Tab 2: cliffy (batch LLM)
- Tab 3: lazygit (version control)
- Tab 4: taskbook (task tracking)

**Optimized Fresh for AI:**
```
Tab 0: CLAUDE (main agent)
  - Full tool access
  - Orchestrates everything
  - Clean context

Tab 1: BUILD
  - Auto-rebuilding
  - Claude checks on-demand
  - Minimal context usage

Tab 2: TEST
  - Test watcher
  - Claude reads failures only
  - Async verification

Tab 3: SERVE
  - Dev server
  - Claude debugs when needed
  - Logs available but not in-context

Tab 4: CODEX (specialized agent)
  - Codebase exploration
  - Can query on-demand
  - Separate context from main Claude

Tab 5: TASKS
  - Task tracking
  - Updated by Claude
  - User views progress

Floating: QUICK TERMINAL
  - One-off commands
  - Doesn't pollute other tabs
```

**The key changes:**

1. **Tab 1-3 are background processes**, not interactive agents
2. **Claude (Tab 0) orchestrates and monitors**
3. **Selective reading** instead of inline output
4. **State persistence** across interactions
5. **Context efficiency** as primary goal

## Implementation: What Works Today

### I Can Already Do This

**Send commands to panes:**
```bash
tmux send-keys -t session:1 "cargo build" C-m
```

**Read from panes:**
```bash
tmux capture-pane -t session:1 -p -S -20
```

**Check if process is running:**
```bash
tmux list-panes -t session:1 -F "#{pane_pid} #{pane_current_command}"
```

**Kill processes in panes:**
```bash
tmux send-keys -t session:1 C-c
```

**Create new panes programmatically:**
```bash
tmux split-window -t session:0 -v
```

### Example: Build Monitor

```bash
# Start build in pane 1
tmux send-keys -t session:1 "cargo build 2>&1 | tee /tmp/build.log" C-m

# Monitor in a loop
while true; do
  # Check last line
  last_line=$(tmux capture-pane -t session:1 -p | tail -1)

  if [[ "$last_line" =~ "Finished" ]]; then
    echo "Build complete"
    break
  elif [[ "$last_line" =~ "error" ]]; then
    echo "Build failed"
    # Read error context
    tmux capture-pane -t session:1 -p -S -50
    break
  fi

  sleep 2
done
```

This script:
- Runs in background
- Monitors build asynchronously
- Only reads full context on error
- Saves massive context space

### Example: Multi-Service Startup

```bash
# Start all services
tmux send-keys -t session:1 "docker-compose up database" C-m
sleep 3
tmux send-keys -t session:2 "npm run dev:backend" C-m
sleep 5
tmux send-keys -t session:3 "npm run dev:frontend" C-m

# Verify all started
for pane in 1 2 3; do
  output=$(tmux capture-pane -t session:$pane -p | tail -5)
  echo "Pane $pane status: $output"
done

# If all good, report ready
# If any failed, read more context
```

All services start in parallel, verification is lightweight.

## Tmux vs Zellij for AI Workflows

### Tmux Advantages for AI

1. **Highly scriptable** - Every operation has a CLI command
2. **Mature** - Well-documented, stable behavior
3. **Parseable output** - Easy to extract info programmatically
4. **Precise control** - Can target specific panes, read specific ranges

**Example:**
```bash
# Read lines 50-100 from pane 2
tmux capture-pane -t session:2 -p -S -100 -E -50

# Get current command in pane
tmux display-message -p -t session:2 "#{pane_current_command}"

# Check if pane is active
tmux display-message -p -t session:2 "#{pane_active}"
```

This level of control is **perfect for AI automation**.

### Zellij for AI

Zellij has CLI actions (`zellij action`) but:
- Less mature than tmux
- Fewer examples online
- Still evolving

**But it might be good enough:**
```bash
# Zellij CLI actions
zellij action write-chars "cargo build"
zellij action write 10  # Enter key

# Read output... (need to research this)
```

**Unknown**: Can Zellij capture pane output programmatically?

If yes: Zellij could work.
If no: Tmux is better for AI workflows.

### The Decision Criteria for AI

**Choose Tmux if:**
- You want maximum scriptability
- You need precise control over panes
- You want proven, stable automation
- You'll write scripts to orchestrate panes

**Choose Zellij if:**
- CLI actions are sufficient
- You prefer better human UX (floating panes)
- You're willing to adapt as it matures
- The layout system matters more than scriptability

**For AI workflows specifically, Tmux has the edge right now.**

But Zellij could catch up if they prioritize CLI/programmatic access.

## The Future: True Multi-Agent Coordination

### What Would Ideal Look Like?

**A protocol for Claude instances to coordinate:**

```yaml
# agent-config.yaml
session: my-project

agents:
  - name: architect
    tab: 0
    role: main coordinator
    model: claude-sonnet-4.5
    tools: [read, write, edit, bash]

  - name: tester
    tab: 1
    role: test runner
    model: claude-haiku  # cheaper model
    tools: [bash, read]
    watch: npm test

  - name: builder
    tab: 2
    role: build monitor
    model: claude-haiku
    tools: [bash]
    watch: cargo build

  - name: docs
    tab: 3
    role: documentation
    model: claude-sonnet
    tools: [read, write]
```

**Agents communicate via shared channel:**

Architect → Tester: "Run tests for authentication module"
Tester → Architect: "3 tests failing: [details]"
Architect: Fixes code
Architect → Tester: "Rerun tests"
Tester → Architect: "All passing"

### The API This Would Need

```python
# Pseudo-code for multi-agent system

class MultiAgentWorkspace:
    def spawn_agent(self, name, role, tab):
        """Spawn a Claude instance in a specific tab"""

    def send_message(self, from_agent, to_agent, message):
        """Agent-to-agent communication"""

    def broadcast(self, from_agent, message):
        """Send to all agents"""

    def query_agent(self, agent, question):
        """Synchronous query, wait for response"""

    def get_agent_state(self, agent):
        """What is agent currently doing?"""
```

**This doesn't exist yet**, but the building blocks do:
- Tmux/Zellij for isolation
- Claude API for multiple instances
- Bash tools for inter-pane communication

Someone could build this.

### Simpler Version: Agent Scripts

Even without full multi-agent protocol, you can have **specialized scripts**:

```bash
# test-agent.sh (runs in Tab 1)
while true; do
  npm test 2>&1 | tee /tmp/test-output.txt

  # If failures, create a report
  if grep -q "FAIL" /tmp/test-output.txt; then
    grep -A 10 "FAIL" /tmp/test-output.txt > /tmp/test-failures.txt
    echo "FAILURES_DETECTED" > /tmp/test-status.txt
  else
    echo "ALL_PASSING" > /tmp/test-status.txt
  fi

  sleep 10
done
```

**Main Claude can read status:**
```bash
status=$(cat /tmp/test-status.txt)
if [[ "$status" == "FAILURES_DETECTED" ]]; then
  failures=$(cat /tmp/test-failures.txt)
  # Fix the failures...
fi
```

This is primitive multi-agent: specialized process + main coordinator.

## Recommendations

### For Your Fresh Setup

**Immediate optimization (works today):**

1. **Keep Tab 0 as main Claude** (where you interact)

2. **Change Tabs 1-3 to background processes:**
   - Tab 1: `cargo watch -x build` (or relevant build)
   - Tab 2: `npm run test:watch` (or relevant tests)
   - Tab 3: Dev server / running app

3. **I monitor tabs via tmux commands:**
   ```bash
   # Check if build succeeded
   tmux capture-pane -t session:1 -p | tail -3 | grep "Finished"

   # Check test status
   tmux capture-pane -t session:2 -p | grep "test result"

   # Check server errors
   tmux capture-pane -t session:3 -p | grep -i error
   ```

4. **Context savings: ~80-90%** on long-running processes

### What Changes in Your Workflow

**Before (inline execution):**
You: "Run the tests"
Me: `npm test` → 5,000 lines of output → all in context

**After (background monitoring):**
You: "Run the tests"
Me: `tmux send-keys -t session:2 "npm test" C-m`
Me: (10 seconds later) `tmux capture-pane -t session:2 -p | grep "test result"`
Output: "test result: ok. 47 passed"
Context used: 15 tokens instead of 5,000

### What You Need to Do

**Minimal changes to fresh:**

1. Update window commands to start watchers:
   ```bash
   # Instead of just launching claude
   # Launch background processes in other windows

   window_cmds=(
     "${TMUX_CLAUDE_CMD:-claude}"
     "cargo watch -x build"  # Tab 1: build
     "npm run test:watch"    # Tab 2: test
     "npm run dev"           # Tab 3: server
     "${TMUX_TASK_CMD:-tb}"  # Tab 4: tasks
   )
   ```

2. I use tmux commands to monitor them

3. Context savings are automatic

**That's it. Huge efficiency gain for minimal work.**

## Conclusion

Terminal multiplexers aren't just for organizing YOUR terminal sessions.

They're **state management and process isolation for AI coding agents**.

By using panes/tabs for long-running processes, and only sampling their output when needed, I can:

1. **Save 80-90% of context** on build/test/server output
2. **Run multiple things in parallel** without blocking
3. **Monitor asynchronously** instead of polling
4. **Maintain persistent state** across interactions
5. **Potentially coordinate** with other AI agents

Your `fresh` setup is already most of the way there. With minor tweaks to launch background processes instead of interactive tools, it becomes an **AI-optimized mission control center**.

**The future of AI-assisted development isn't just better AI.**

It's better interfaces between AI and the development environment.

Terminal multiplexers are that interface.
