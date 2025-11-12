# Fresh Hybrid Implementation Plan

## Vision

**Single command to rule your development workspace:**

```bash
# Launch your main dashboard
fresh

# In any project directory
cd ~/projects/my-app
fresh init  # Adds project as new Zellij tab with tmux workspace
```

**What you see:**
```
┌─────────────────────────────────────────────────────┐
│ Zellij Dashboard                                    │
├─────────────────────────────────────────────────────┤
│ [home] [dotfiles] [web-app] [ai-project] [+]       │
│                                                     │
│ Current tab: dotfiles                               │
│ ┌─────────────────────────────────────────────────┐ │
│ │ claude@dotfiles:~$                              │ │
│ │ > Let's implement user authentication            │ │
│ │                                                  │ │
│ │ (tmux running in background with build/test)    │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│ [Ctrl-T] Toggle taskbook                            │
└─────────────────────────────────────────────────────┘
```

## Architecture

### Layer 1: Zellij (User Interface)
- **Dashboard tab**: Home base, quick commands, overview
- **Project tabs**: One per project (dotfiles, web-app, etc.)
- **Floating taskbook**: Ctrl-T toggles across all tabs
- **Quick terminal**: Floating pane for one-off commands

### Layer 2: Tmux (AI Automation)
- **Inside each project tab**: Tmux session
- **Windows**: claude (main), build, test, server
- **Claude monitors**: Background windows via tmux CLI
- **You never see**: The tmux layer (invisible)

### Commands

```bash
fresh              # Launch/attach to Zellij dashboard
fresh init         # Add current project to dashboard
fresh remove       # Remove current project from dashboard
fresh list         # List all projects in dashboard
fresh kill         # Kill current project's workspace
```

## Implementation Plan

### Phase 1: File Structure (5 minutes)

```
dotfiles/
  bin/
    fresh                          # Main entry point script
  config/
    zellij/
      config.kdl                   # Zellij config with keybindings
      layouts/
        dashboard.kdl              # Dashboard layout
        project.kdl                # Project tab template
  scripts/
    fresh/
      init-project.sh              # Initialize project in dashboard
      remove-project.sh            # Remove project
      tmux-setup.sh                # Set up tmux windows
```

### Phase 2: Core Scripts (30 minutes)

#### 2.1: Main Entry Point (`bin/fresh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

FRESH_DIR="$HOME/.fresh"
PROJECTS_FILE="$FRESH_DIR/projects.txt"

# Initialize fresh directory
init_fresh() {
    mkdir -p "$FRESH_DIR"
    touch "$PROJECTS_FILE"
}

# Main command: launch or attach to dashboard
main() {
    init_fresh

    # Check if dashboard session exists
    if zellij list-sessions 2>/dev/null | grep -q "^dashboard"; then
        exec zellij attach dashboard
    else
        exec zellij --session dashboard --layout dashboard
    fi
}

# Subcommands
case "${1:-}" in
    init)
        shift
        exec "$HOME/dotfiles/scripts/fresh/init-project.sh" "$@"
        ;;
    remove)
        shift
        exec "$HOME/dotfiles/scripts/fresh/remove-project.sh" "$@"
        ;;
    list)
        cat "$PROJECTS_FILE"
        ;;
    kill)
        zellij action close-tab
        ;;
    *)
        main
        ;;
esac
```

#### 2.2: Project Initialization (`scripts/fresh/init-project.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR=$(pwd)
PROJECT_NAME=$(basename "$PROJECT_DIR" | sed 's/[^a-zA-Z0-9_-]/_/g')
PROJECTS_FILE="$HOME/.fresh/projects.txt"

# Add to projects list
echo "$PROJECT_NAME|$PROJECT_DIR" >> "$PROJECTS_FILE"

# Attach to dashboard session
zellij action new-tab --layout project --name "$PROJECT_NAME"

# Inside the new tab, a tmux session will be created
# (handled by project.kdl layout)

echo "✓ Added '$PROJECT_NAME' to dashboard"
echo "  Switch to it with Alt-Tab or Ctrl-g + Tab menu"
```

#### 2.3: Tmux Workspace Setup (`scripts/fresh/tmux-setup.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

# This script runs inside the Zellij project tab
# It creates the tmux workspace for Claude

SESSION_NAME="${1:-claude-work}"
PROJECT_DIR="${2:-$(pwd)}"

# Create tmux session with claude in window 0
tmux new-session -d -s "$SESSION_NAME" -n claude -c "$PROJECT_DIR"

# Window 1: Build watcher
tmux new-window -t "$SESSION_NAME" -n build -c "$PROJECT_DIR"
tmux send-keys -t "$SESSION_NAME:build" "# Ready for build commands" C-m

# Window 2: Test watcher
tmux new-window -t "$SESSION_NAME" -n test -c "$PROJECT_DIR"
tmux send-keys -t "$SESSION_NAME:test" "# Ready for test commands" C-m

# Window 3: Dev server
tmux new-window -t "$SESSION_NAME" -n server -c "$PROJECT_DIR"
tmux send-keys -t "$SESSION_NAME:server" "# Ready for server commands" C-m

# Select main window and attach
tmux select-window -t "$SESSION_NAME:claude"
exec tmux attach -t "$SESSION_NAME"
```

### Phase 3: Zellij Layouts (20 minutes)

#### 3.1: Dashboard Layout (`config/zellij/layouts/dashboard.kdl`)

```kdl
layout {
    default_tab_template {
        pane size=1 borderless=true {
            plugin location="zellij:tab-bar"
        }
        children
        pane size=2 borderless=true {
            plugin location="zellij:status-bar"
        }
    }

    tab name="home" focus=true {
        pane {
            command "bash"
            args "-c" "clear && cat ~/.fresh/welcome.txt || echo 'Welcome to Fresh Dashboard'"
        }
    }

    // Floating taskbook pane (Ctrl-T to toggle)
    floating_panes {
        pane name="tasks" {
            command "bash"
            args "-c" "while true; do clear; tb 2>/dev/null || echo 'No tasks'; sleep 2; done"
            x "25%"
            y "10%"
            width "50%"
            height "80%"
        }
    }
}
```

#### 3.2: Project Layout (`config/zellij/layouts/project.kdl`)

```kdl
layout {
    default_tab_template {
        pane size=1 borderless=true {
            plugin location="zellij:tab-bar"
        }
        children
        pane size=2 borderless=true {
            plugin location="zellij:status-bar"
        }
    }

    tab {
        pane {
            // This launches the tmux setup script
            // Which creates tmux session and attaches
            command "bash"
            args "-c" "~/.config/zellij/scripts/tmux-setup.sh"
        }
    }
}
```

#### 3.3: Zellij Config (`config/zellij/config.kdl`)

```kdl
keybinds {
    normal {
        // Toggle taskbook
        bind "Ctrl t" { ToggleFloatingPanes; }

        // Quick tab switching
        bind "Alt 1" { GoToTab 1; }
        bind "Alt 2" { GoToTab 2; }
        bind "Alt 3" { GoToTab 3; }
        bind "Alt 4" { GoToTab 4; }
        bind "Alt 5" { GoToTab 5; }

        // New project tab (prompts for name)
        bind "Alt n" { NewTab; }
    }
}

// Copy on select
copy_on_select true

// Mouse mode
mouse_mode true

// Simplified UI
pane_frames false
```

### Phase 4: Testing Checklist (15 minutes)

```bash
# Test 1: Launch dashboard
fresh
# Expected: Zellij opens with home tab

# Test 2: Add a project
cd ~/dotfiles
fresh init
# Expected: New "dotfiles" tab appears, tmux session inside

# Test 3: Verify tmux inside Zellij
# In the dotfiles tab, verify:
tmux list-windows
# Expected: claude, build, test, server windows

# Test 4: Toggle taskbook
# Press Ctrl-T
# Expected: Taskbook floats over current view
# Press Ctrl-T again
# Expected: Taskbook hides

# Test 5: Add another project
cd ~/projects/web-app
fresh init
# Expected: New "web-app" tab appears

# Test 6: Switch between projects
# Press Alt-1, Alt-2
# Expected: Switches between tabs

# Test 7: Claude monitoring
# From claude window (window 0), send command to build window:
tmux send-keys -t build "echo 'test build'" C-m
tmux capture-pane -t build -p | tail -5
# Expected: See "test build" in output

# Test 8: List projects
fresh list
# Expected: Shows dotfiles and web-app

# Test 9: Reattach to dashboard
# Detach from Zellij (Ctrl-g, d)
fresh
# Expected: Reattaches to existing session, all tabs still there
```

## Phase 5: Polish (20 minutes)

### 5.1: Welcome Screen

```bash
# ~/.fresh/welcome.txt
╔══════════════════════════════════════════════════════╗
║              Fresh Development Dashboard             ║
╚══════════════════════════════════════════════════════╝

Quick Commands:
  fresh init     - Add current directory as project tab
  fresh list     - List all projects
  fresh remove   - Remove current project

Shortcuts:
  Ctrl-T         - Toggle taskbook
  Alt-1..9       - Switch to tab 1-9
  Alt-N          - New tab
  Ctrl-G + D     - Detach from dashboard

Projects in dashboard:
$(cat ~/.fresh/projects.txt 2>/dev/null | awk -F'|' '{print "  •", $1}')

Ready to code!
```

### 5.2: Project Detection

Add to `scripts/fresh/tmux-setup.sh`:

```bash
# Auto-detect project type and set up watchers

detect_project_type() {
    local project_dir=$1

    if [[ -f "$project_dir/Cargo.toml" ]]; then
        echo "rust"
    elif [[ -f "$project_dir/package.json" ]]; then
        echo "node"
    elif [[ -f "$project_dir/requirements.txt" ]] || [[ -f "$project_dir/pyproject.toml" ]]; then
        echo "python"
    elif [[ -f "$project_dir/go.mod" ]]; then
        echo "go"
    else
        echo "generic"
    fi
}

setup_watchers() {
    local project_type=$1
    local session=$2

    case $project_type in
        rust)
            tmux send-keys -t "$session:build" "cargo watch -x build" C-m
            tmux send-keys -t "$session:test" "cargo watch -x test" C-m
            ;;
        node)
            tmux send-keys -t "$session:build" "npm run build:watch 2>/dev/null || echo 'No build:watch script'" C-m
            tmux send-keys -t "$session:test" "npm run test:watch 2>/dev/null || npm test -- --watch" C-m
            tmux send-keys -t "$session:server" "npm run dev 2>/dev/null || npm start" C-m
            ;;
        python)
            tmux send-keys -t "$session:test" "ptw 2>/dev/null || pytest-watch" C-m
            tmux send-keys -t "$session:server" "python manage.py runserver 2>/dev/null || uvicorn main:app --reload" C-m
            ;;
        go)
            tmux send-keys -t "$session:build" "air 2>/dev/null || go run ." C-m
            tmux send-keys -t "$session:test" "go test -v ./..." C-m
            ;;
    esac
}

PROJECT_TYPE=$(detect_project_type "$PROJECT_DIR")
setup_watchers "$PROJECT_TYPE" "$SESSION_NAME"
```

## Success Criteria

After implementation, you should be able to:

1. ✅ Run `fresh` to launch your development dashboard
2. ✅ Navigate between projects with Alt-1, Alt-2, etc.
3. ✅ Toggle taskbook with Ctrl-T from any tab
4. ✅ Run `fresh init` in a new project to add it
5. ✅ Have tmux automatically set up in each project tab
6. ✅ See auto-detected build/test watchers running
7. ✅ Claude can monitor background processes efficiently
8. ✅ Detach and reattach without losing state
9. ✅ Beautiful, modern Zellij UI
10. ✅ Context-efficient tmux automation in background

## Timeline

- **Phase 1 (File Structure):** 5 minutes
- **Phase 2 (Core Scripts):** 30 minutes
- **Phase 3 (Zellij Layouts):** 20 minutes
- **Phase 4 (Testing):** 15 minutes
- **Phase 5 (Polish):** 20 minutes

**Total: ~90 minutes to working system**

---

# Unstructured Future Ideas 🚀

## Wild Ideas to Explore

### 1. Project Persistence & State
```bash
# Save entire workspace state
fresh save
# Creates snapshot: all tabs, all tmux windows, running commands

# Restore later (even after reboot)
fresh restore
# Recreates exact state: projects, commands, positions
```

**Implementation:**
- Save tmux layouts: `tmux list-windows -F "#{window_layout}"`
- Save running commands: `tmux capture-pane -p` for each
- Store in `~/.fresh/state.json`
- Restore with `tmux respawn-pane -k -t`

### 2. AI Agent Coordination Protocol

**Multiple Claude instances coordinating:**

```bash
fresh spawn agent --role tester
# Creates new tab with dedicated Claude instance
# Role: Monitor tests, report failures
# Communicates with main Claude via shared files

fresh spawn agent --role builder
# Dedicated to builds
# Reports: "Build complete" or "Build failed: [error]"
```

**Communication:**
```bash
~/.fresh/agents/
  main.inbox      # Messages to main Claude
  tester.inbox    # Messages to tester agent
  builder.inbox   # Messages to builder agent
```

Main Claude:
```bash
echo "run-tests" > ~/.fresh/agents/tester.inbox
# Tester agent picks it up, runs tests, writes result
cat ~/.fresh/agents/main.inbox
# "tests-passed: 47/47"
```

### 3. Context Streaming

**Problem:** Even with selective monitoring, context fills up.

**Solution:** Stream context to long-term storage, recall on-demand.

```bash
# As I work, stream thoughts to file
~/.fresh/projects/dotfiles/
  session-2025-10-17.md  # Full conversation log
  decisions.md           # Key decisions made
  todo-history.md        # Task evolution

# Later, recall context
"What did we decide about the authentication flow?"
# I search session-2025-10-15.md, find decision, continue
```

**Enables multi-day projects without losing context.**

### 4. Visual Task Dashboard

**Instead of text taskbook, web-based dashboard:**

```bash
fresh dashboard
# Opens localhost:3333 with live dashboard

Features:
- Current tasks (from taskbook)
- Build/test status (from tmux panes)
- Recent errors (from log scraping)
- Project graph (dependencies between projects)
- Time tracking per task
- AI activity feed ("Claude is refactoring auth.rs...")
```

**Implementation:**
- Lightweight web server in background tab
- WebSocket to Zellij/tmux for real-time updates
- React dashboard (because why not)

### 5. Project Templates

```bash
fresh init --template rust-web-api
# Sets up:
# - Cargo workspace
# - Docker compose
# - GitHub Actions
# - Tmux windows preconfigured for Rust
# - Pre-populated tasks

fresh init --template nextjs-app
# Full Next.js stack
# - Frontend/backend split
# - Prisma database
# - Tailwind
# - Tmux windows for dev server, db, tests
```

**Templates stored in:**
```
~/.fresh/templates/
  rust-web-api/
    Cargo.toml
    docker-compose.yml
    .github/workflows/ci.yml
    .fresh-config.sh  # Custom tmux setup
```

### 6. Session Recording & Replay

**Record entire development sessions:**

```bash
fresh record
# Starts recording:
# - All commands
# - All file edits
# - All AI interactions
# - Timestamps

fresh replay session-2025-10-17
# Plays back in fast-forward
# Shows: "10:32 - Created user.rs"
# Shows: "10:45 - Fixed auth bug"
# Shows: "11:20 - Tests passing"
```

**Use case:**
- Onboarding ("here's how we built this feature")
- Debugging ("what changed between when it worked and now?")
- Learning ("replay expert's workflow")

### 7. Multi-Machine Sync

**Sync your fresh workspace across machines:**

```bash
# On laptop
fresh push
# Uploads workspace state to cloud

# On desktop
fresh pull
# Downloads workspace, recreates tabs, reconnects
```

**Enables:**
- Start on laptop, continue on desktop
- Multiple developers sharing workspace
- Disaster recovery (laptop dies, desktop has everything)

### 8. Smart Context Windows

**AI-driven context management:**

I monitor my own context usage:
- When at 80% capacity: "Compacting old conversation..."
- Preserve: Recent code, active decisions, current task
- Archive: Old build logs, completed todos, past errors
- Store: Everything in `~/.fresh/projects/X/context-archive/`

When you ask: "Why did we use Redis?"
- I search archived context
- Recall the decision
- Continue without "I don't remember, context limit"

### 9. Team Collaboration Mode

**Multiple humans, multiple AIs, one workspace:**

```bash
fresh share
# Creates shareable workspace URL
# Other developer joins

# You see:
Tab 1: Your Claude (main work)
Tab 2: Alice's Claude (working on tests)
Tab 3: Bob's Claude (working on docs)

# Shared floating pane: Team tasks
# Each agent updates progress
# Coordination happens automatically
```

**AI agents negotiate:**
- "I'm working on auth.rs, Alice's Claude is working on user.rs"
- "Bob's Claude finished docs, ready to merge"
- "Build is failing, Test Claude is investigating"

### 10. Voice Control

**Talk to your workspace:**

```
"Hey Fresh, show me the build status"
→ Switches to build tab, shows last 20 lines

"Run the tests"
→ Sends "cargo test" to test window

"What's Claude working on?"
→ Shows current task from taskbook

"Add a task: implement rate limiting"
→ Creates task in taskbook
```

**Implementation:**
- Background process with STT (Speech-to-Text)
- Maps voice commands to zellij/tmux actions
- TTS feedback: "Tests are passing"

### 11. Metrics & Analytics

**Track your development patterns:**

```bash
fresh stats
# Shows:
# - Most active projects
# - Build success rate
# - Test frequency
# - Time spent per task
# - Claude context usage trends
# - Lines of code written (per project)

fresh insights
# AI analysis:
# "You spend 40% of time on refactoring"
# "Tests fail most often on Fridays" (need more sleep?)
# "Auth.rs has the most churn"
# "Suggestion: Break user.rs into smaller modules"
```

### 12. Pluggable Tools

**Fresh as a platform:**

```bash
fresh plugin install fresh-llm-router
# Adds command: fresh llm <prompt>
# Routes to best model (Claude for code, GPT for creative, etc.)

fresh plugin install fresh-git-butler
# Adds smart git automation
# "fresh commit" → AI writes commit message from diff

fresh plugin install fresh-deploy
# Adds deployment integration
# "fresh deploy staging" → deploys current branch
```

**Plugin API:**
```bash
# Plugins are just bash scripts in ~/.fresh/plugins/
# They get access to:
# - FRESH_PROJECT_DIR
# - FRESH_SESSION_NAME
# - Tmux session controls
# - Zellij action commands
```

### 13. Time Travel Debugging

**Snapshot workspace state every 5 minutes:**

```bash
fresh snapshots
# Lists: 10:30, 10:35, 10:40, 10:45...

fresh goto 10:35
# Restores:
# - File contents at 10:35
# - Tmux window state
# - Running processes
# - Current task

# Debug: "It worked at 10:35, broke by 10:45"
# Diff the two snapshots, find the breaking change
```

### 14. AI Pair Programming Modes

**Different AI personalities for different tasks:**

```bash
fresh mode strict
# Claude becomes strict code reviewer
# Rejects anything not following best practices
# "This function is too long. Refactor."

fresh mode creative
# Claude suggests wild ideas
# "What if we used WebAssembly here?"

fresh mode tutorial
# Claude explains everything in detail
# Good for learning new tech

fresh mode speed
# Claude prioritizes fast iteration over perfection
# Good for prototyping
```

### 15. Cross-Project Intelligence

**AI learns across all your projects:**

```bash
# In dotfiles project
You: "How did I handle config files in web-app?"

# Claude searches web-app project context
# Finds: "You used dotenv with type-safe wrappers"
# Suggests: "Want me to use the same pattern here?"
```

**Shared knowledge base:**
```
~/.fresh/knowledge/
  patterns.md     # Patterns you use across projects
  decisions.md    # Architectural decisions
  snippets/       # Reusable code snippets
  gotchas.md      # Things that tripped you up
```

AI builds this automatically as you work.

## The Meta Idea: Fresh as Operating System

**Eventually, Fresh becomes your entire development OS:**

- Projects are "apps"
- Tmux windows are "processes"
- Taskbook is "window manager"
- Claude is "kernel" (coordinates everything)
- Plugins are "system calls"

You never leave Fresh. It's your environment.

**Boot sequence:**
1. Open terminal
2. `fresh`
3. Everything you need is there
4. Projects, tools, AI, tasks
5. Code until done
6. `Ctrl-G, D` to suspend
7. `fresh` tomorrow to resume

**The workspace that persists.**

## Implementation Priority

**Now (get it working):**
1. Basic fresh command
2. Zellij + tmux nesting
3. Project init/remove
4. Taskbook floating pane

**Soon (make it awesome):**
1. Auto-detect project type
2. Smart tmux setup per language
3. Context streaming
4. Session persistence

**Later (make it magical):**
1. AI agent coordination
2. Multi-machine sync
3. Visual dashboard
4. Voice control

**Eventually (make it sci-fi):**
1. Time travel debugging
2. Cross-project intelligence
3. Team collaboration mode
4. Fresh as OS

---

Ready to build Phase 1?
