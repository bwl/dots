# RETRO-OS: A Forever-Running Personal Operating Environment

**Vision**: A single, permanent tmux session that feels like a complete operating system—retro aesthetics meet modern power tools, with a command palette at the heart of everything.

---

## Table of Contents

1. [Philosophy](#philosophy)
2. [Core Architecture](#core-architecture)
3. [The Permanent Layout](#the-permanent-layout)
4. [Command Palette](#command-palette)
5. [Pod System](#pod-system)
6. [Boot Sequence](#boot-sequence)
7. [Navigation](#navigation)
8. [Retro Aesthetics](#retro-aesthetics)
9. [Technical Implementation](#technical-implementation)
10. [Implementation Roadmap](#implementation-roadmap)

---

## Philosophy

### Core Principles

**1. One Session, Forever**
- Single tmux session that never dies
- Survives reboots via tmux-resurrect
- Your entire digital environment in one place
- No window juggling, no session management

**2. Command Palette First**
- `Ctrl+Space` → do anything
- Type to search, fuzzy find, execute
- No memorizing window numbers or layouts
- Like Raycast/Alfred but for your terminal OS

**3. Pods, Not Windows**
- Tools spawn on-demand in the workspace
- Dismiss when done (Esc or Cmd+W)
- No pre-allocated windows sitting empty
- Fluid, invoke-and-dismiss workflow

**4. Persistent Dashboard**
- Always-visible sidebars with context
- Left: Quick actions, tasks, shortcuts
- Right: Calendar, system, notifications
- Center: Workspace where you actually work

**5. Retro Soul, Modern Power**
- CRT shaders, retro fonts, boot sequences
- But with AI, fuzzy search, cloud sync
- Feels like Commodore 64, performs like 2025

**6. Intentional Desktop Escape**
- Everything possible in terminal
- Desktop apps are escape hatches (Raycast)
- Return to RETRO-OS is always one keystroke

---

## Core Architecture

### The Forever Session

```
┌─────────────────────────────────────────────────────────────┐
│                         RETRO-OS                            │
│                    (Single tmux session)                    │
├───────────┬─────────────────────────────────┬───────────────┤
│           │                                 │               │
│  SIDEBAR  │       MAIN WORKSPACE            │  INFO PANEL   │
│           │                                 │               │
│  • Tasks  │   [Active Pod or Dashboard]     │  • Calendar   │
│  • Quick  │                                 │  • System     │
│    Actions│   Default: Dashboard            │  • Weather    │
│  • Recent │   Pods: Email, Files, Editor,   │  • Notifs     │
│           │         Music, Monitor, etc.    │               │
│           │                                 │               │
│           │   Ctrl+Space → Summon anything  │               │
│           │                                 │               │
├───────────┴─────────────────────────────────┴───────────────┤
│  [Session: master] [Time: 2:34 PM] [Battery: 87%] [WiFi ✓] │
└─────────────────────────────────────────────────────────────┘
```

### How It Works

**Persistent Layout** (3 panes in 1 window):
1. **Left Sidebar** (15%): Tasks, actions, navigation
2. **Main Workspace** (60%): Where pods appear
3. **Right Panel** (25%): Calendar, system info

**Pod Invocation**:
```
You: Ctrl+Space
System: Opens command palette (fzf)
You: Type "email"
System: Spawns email client in main workspace
You: Check email, reply
You: Press Esc or Cmd+W
System: Returns to dashboard
```

**No Window Switching**:
- Everything happens in one window
- Pods replace each other in workspace
- Or spawn as popups (overlay)
- Or split as panels (music player at bottom)

---

## The Permanent Layout

### Full Layout Design

```
┌─────────────────────────────────────────────────────────────┐
│ RETRO-OS v1.0                          2025-10-16  2:34 PM │
├───────────┬─────────────────────────────────┬───────────────┤
│           │                                 │               │
│ QUICK     │      MAIN WORKSPACE             │  TODAY        │
│ ACTIONS   │                                 │  ───────────  │
│ ───────── │  Nothing active = Dashboard     │  9:00 AM      │
│           │                                 │  Standup      │
│ ○ Email   │  ⚡ WELCOME BACK                │               │
│ ○ Files   │                                 │  2:00 PM      │
│ ○ Edit    │  Press Ctrl+Space for actions   │  PR Review    │
│ ○ Music   │                                 │               │
│ ○ Tasks   │  Recent:                        │  ───────────  │
│ ○ Fresh   │  • dotfiles (2m ago)            │               │
│           │  • setup (1h ago)               │  TASKS        │
│ ───────── │  • tarot (yesterday)            │  ───────────  │
│           │                                 │               │
│ RECENT    │  Active Tasks:                  │  ◉ #1 Docs    │
│ ───────── │  ◉ #1 Write docs                │  ◉ #2 Test    │
│           │  ◉ #2 Test pods                 │  ◯ #3 Deploy  │
│ dotfiles  │  ◯ #3 Deploy                    │               │
│ setup     │                                 │  ───────────  │
│ tarot     │  [Ctrl+Space to do anything]    │               │
│           │                                 │  SYSTEM       │
│           │                                 │  ───────────  │
│           │                                 │  CPU: 34%     │
│           │                                 │  Mem: 8/16GB  │
│           │                                 │  Bat: 87%     │
│           │                                 │               │
├───────────┴─────────────────────────────────┴───────────────┤
│  master | Cmd+Space:Raycast | Ctrl+Space:Palette | ?:Help  │
└─────────────────────────────────────────────────────────────┘
```

### Pane Roles

**Left Sidebar** (`sidebar.sh`):
- Quick actions (single-key shortcuts)
- Recent projects
- Favorite commands
- Task list preview
- Always visible, never changes

**Main Workspace** (60%):
- Default: Dashboard welcome screen
- Active pod: Takes over this space
- Popup pod: Overlays on top
- Split pod: Shares space (e.g., music at bottom)

**Right Panel** (`info-panel.sh`):
- Today's calendar
- Active tasks (from taskbook)
- System stats (CPU, memory, battery)
- Weather widget
- Notifications
- Auto-refreshes every 30s

**Status Bar** (bottom):
- Session name
- Current time
- Quick shortcuts reminder
- Active pod indicator

---

## Command Palette

### The Heart of RETRO-OS

**Trigger**: `Ctrl+Space` (anywhere, anytime)

**What It Does**:
- Fuzzy-find actions
- Launch pods
- Quick captures (task, note, etc.)
- System commands
- Git operations
- Fresh session management

### Example Interactions

```
Ctrl+Space → Type: "email"
┌─────────────────────────────────────────┐
│ RETRO-OS Command Palette                │
├─────────────────────────────────────────┤
│ > email                                 │
│                                         │
│  📧 Email Client                        │
│  📨 New Email                           │
│  📬 Check Inbox                         │
│                                         │
└─────────────────────────────────────────┘
Select → Email client spawns in workspace
```

```
Ctrl+Space → Type: "task"
┌─────────────────────────────────────────┐
│ RETRO-OS Command Palette                │
├─────────────────────────────────────────┤
│ > task                                  │
│                                         │
│  ✓ New Task (Quick Capture)            │
│  📋 View All Tasks                      │
│  ✅ Mark Task Done                      │
│  🗑️  Delete Task                        │
│                                         │
└─────────────────────────────────────────┘
Select → Quick capture input appears
```

```
Ctrl+Space → Type: "file README"
┌─────────────────────────────────────────┐
│ RETRO-OS Command Palette                │
├─────────────────────────────────────────┤
│ > file README                           │
│                                         │
│  📄 dotfiles/README.md                  │
│  📄 setup/README.md                     │
│  📄 tarot/README.md                     │
│                                         │
└─────────────────────────────────────────┘
Select → Opens in editor pod
```

### Command Categories

**Pod Launchers**:
- `email` → Email client
- `files` → File browser (yazi)
- `edit <file>` → Editor
- `music` → Music player
- `monitor` → System monitor
- `fresh` → Spawn fresh session
- `notes` → Notes browser

**Quick Captures**:
- `task <text>` → New task
- `note <text>` → Quick note
- `event <text>` → Calendar event

**System Actions**:
- `update` → Update packages
- `backup` → Create backup
- `health` → Health check
- `theme <name>` → Switch theme

**Git Operations**:
- `commit` → Git commit
- `push` → Git push
- `status` → Git status
- `lazygit` → Launch lazygit

**Project Management**:
- `fresh <project>` → Start fresh session
- `project <name>` → Switch project
- `recent` → Recent projects

---

## Pod System

### What are Pods?

Pods are tools/interfaces that spawn on-demand:
- Live in the main workspace
- Summoned via command palette
- Dismissed when done (Esc/Cmd+W)
- State may or may not persist

### Pod Catalog

#### 1. Email Pod

**Command**: `email`

**Spawn Location**: Main workspace (full takeover)

**Tool**: himalaya or neomutt

**Layout**:
```
┌─────────────────────────────────────────┐
│ EMAIL CLIENT (himalaya)                 │
├─────────────────────────────────────────┤
│                                         │
│  Inbox (12 unread)                      │
│  ├─ Work (8)                            │
│  ├─ Personal (3)                        │
│  └─ Newsletters (1)                     │
│                                         │
│  [Reading: Team update from Sarah]      │
│                                         │
│  Actions:                               │
│  r: Reply  f: Forward  d: Delete        │
│  Esc: Return to dashboard               │
│                                         │
└─────────────────────────────────────────┘
```

**Dismiss**: Esc or Cmd+W

---

#### 2. Files Pod

**Command**: `files`

**Spawn Location**: Main workspace or popup

**Tool**: yazi

**Layout** (popup mode):
```
┌─────────────────────────────────────────┐
│ FILES (yazi)                            │
├──────────┬──────────────────────────────┤
│          │                              │
│ Tree     │  Preview                     │
│          │                              │
│ ~/       │  README.md                   │
│ ├dotfiles│  # Dotfiles                  │
│ ├setup   │  My personal macOS...        │
│ └tarot   │                              │
│          │  [Syntax highlighted]        │
│          │                              │
│ hjkl:nav │  Enter:open  /:search        │
└──────────┴──────────────────────────────┘
```

**Dismiss**: q or Esc

---

#### 3. Editor Pod

**Command**: `edit <file>` or just `edit`

**Spawn Location**: Main workspace

**Tool**: neovim, claude, or codex

**Layout**:
```
┌─────────────────────────────────────────┐
│ EDITOR (neovim)                         │
├─────────────────────────────────────────┤
│                                         │
│ docs/RETRO-OS.md                        │
│                                         │
│ # RETRO-OS                              │
│ A forever-running personal OS...        │
│                                         │
│ [Editing...]                            │
│                                         │
│ :w to save, :q to exit                  │
│                                         │
└─────────────────────────────────────────┘
```

**Dismiss**: `:q` or Cmd+W

---

#### 4. Music Pod

**Command**: `music`

**Spawn Location**: Bottom panel (split)

**Tool**: musikcube or cmus

**Layout** (bottom 30% of workspace):
```
┌─────────────────────────────────────────┐
│ [Your main workspace continues above]   │
├─────────────────────────────────────────┤
│ 🎵 NOW PLAYING                          │
│ Radiohead - Paranoid Android            │
│ ▶ ━━━━━●────── 3:42 / 6:23             │
│ [◀◀] [⏸] [▶▶]  Vol:70%  Shuffle:OFF   │
└─────────────────────────────────────────┘
```

**Dismiss**: Cmd+W (closes split, workspace expands)

---

#### 5. Monitor Pod

**Command**: `monitor` or `htop`

**Spawn Location**: Main workspace or popup

**Tool**: htop, btop, or custom dashboard

**Layout**:
```
┌─────────────────────────────────────────┐
│ SYSTEM MONITOR                          │
├─────────────────────────────────────────┤
│                                         │
│ CPU: [████████░░] 45%                   │
│ Mem: [███████░░░] 8.2 / 16 GB           │
│ Disk: [███░░░░░░░] 234 / 512 GB         │
│                                         │
│ TOP PROCESSES:                          │
│ PID   NAME        CPU    MEM            │
│ 1234  claude      23%    1.2GB          │
│ 5678  tmux        12%    0.8GB          │
│                                         │
│ q:Quit  k:Kill process                  │
└─────────────────────────────────────────┘
```

**Dismiss**: q

---

#### 6. Fresh Pod

**Command**: `fresh` or `fresh <project>`

**Spawn Location**: New tmux window (separate from RETRO-OS)

**Tool**: Your existing fresh command

**Behavior**:
- Spawns in a new tmux window
- Full fresh session (claude, codex, git, tasks, etc.)
- Cmd+0 returns to RETRO-OS
- Cmd+1 goes to fresh session

**Integration**:
```
RETRO-OS (window 0) ←→ Fresh Session (window 1)
     Permanent              Project-specific
```

---

#### 7. Notes Pod

**Command**: `notes` or `note`

**Spawn Location**: Main workspace

**Tool**: nvim + markdown or Obsidian bridge

**Layout**:
```
┌──────────┬──────────────────────────────┐
│          │                              │
│ Notes    │  # Daily Note 2025-10-16     │
│          │                              │
│ Daily    │  ## Morning                  │
│ Work     │  - RETRO-OS design           │
│ Personal │  - Pod system architecture   │
│ Projects │                              │
│          │  ## Ideas                    │
│ Recent   │  - Command palette is key    │
│ • 10-16  │  - Pods > windows            │
│ • 10-15  │                              │
│          │  [Writing...]                │
│          │                              │
└──────────┴──────────────────────────────┘
```

**Quick Capture**: `Ctrl+Space` → `note Quick idea here` → Appends to today's note

**Dismiss**: :q or Cmd+W

---

#### 8. Calendar Pod

**Command**: `calendar` or `cal`

**Spawn Location**: Popup overlay

**Tool**: calcurse or khal

**Layout**:
```
┌─────────────────────────────────────────┐
│ CALENDAR                                │
├─────────────────────────────────────────┤
│                                         │
│    October 2025                         │
│  S  M  T  W  T  F  S                    │
│           1  2  3  4  5                 │
│  6  7  8  9 10 11 12                    │
│ 13 14 15 ●16 17 18 19                   │
│ 20 21 22 23 24 25 26                    │
│ 27 28 29 30 31                          │
│                                         │
│ TODAY (Oct 16):                         │
│ • 9:00 AM - Team Standup                │
│ • 2:00 PM - PR Review                   │
│                                         │
│ n:New event  d:Delete  q:Close          │
└─────────────────────────────────────────┘
```

**Dismiss**: q or Esc

---

#### 9. Task Pod

**Command**: `tasks`

**Spawn Location**: Main workspace

**Tool**: taskbook with full interface

**Layout**:
```
┌─────────────────────────────────────────┐
│ TASKS (taskbook)                        │
├─────────────────────────────────────────┤
│                                         │
│ TODAY (3 tasks)                         │
│ ◉ 1. Write RETRO-OS docs                │
│ ◉ 2. Test pod system                    │
│ ◯ 3. Deploy to production               │
│                                         │
│ THIS WEEK (7 tasks)                     │
│ ◯ 4. Blog post about RETRO-OS           │
│ ◯ 5. Video demo                         │
│ ...                                     │
│                                         │
│ Commands:                               │
│ t:New  c:Complete  d:Delete  f:Filter   │
└─────────────────────────────────────────┘
```

**Quick Add**: `Ctrl+Space` → `task Write blog post` → Adds immediately

**Dismiss**: q or Cmd+W

---

#### 10. Admin Pod

**Command**: `admin` or `settings`

**Spawn Location**: Main workspace

**Tool**: Custom TUI

**Layout**:
```
┌─────────────────────────────────────────┐
│ RETRO-OS ADMIN                          │
├─────────────────────────────────────────┤
│                                         │
│ QUICK ACTIONS                           │
│ [1] Update all packages                 │
│ [2] Run health check                    │
│ [3] Create backup                       │
│ [4] Switch theme                        │
│ [5] Switch profile                      │
│                                         │
│ SYSTEM STATUS                           │
│ ✓ All packages up to date               │
│ ⚠ Shell startup: 287ms (slow)           │
│ ✓ Bitwarden unlocked                    │
│ ✓ Syncthing synced                      │
│                                         │
│ Press number to run action              │
└─────────────────────────────────────────┘
```

**Dismiss**: q or Esc

---

### Pod Behaviors

**Full Takeover** (email, files, editor, notes):
- Replaces main workspace entirely
- Sidebars remain visible
- Esc returns to dashboard

**Popup Overlay** (calendar, quick actions):
- Appears in center as overlay
- Dismisses with q or Esc
- Dashboard visible underneath

**Bottom Panel** (music, logs):
- Splits workspace horizontally
- Main work continues above
- Close with Cmd+W

**New Window** (fresh sessions):
- Spawns separate tmux window
- Navigate with Cmd+0/1
- Independent of RETRO-OS layout

---

## Boot Sequence

### Visual Design

```
┌─────────────────────────────────────────────────────────────┐
│                                                               │
│           ██████╗ ███████╗████████╗██████╗  ██████╗          │
│           ██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██╔═══██╗         │
│           ██████╔╝█████╗     ██║   ██████╔╝██║   ██║         │
│           ██╔══██╗██╔══╝     ██║   ██╔══██╗██║   ██║         │
│           ██║  ██║███████╗   ██║   ██║  ██║╚██████╔╝         │
│           ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝          │
│                          OS v1.0                              │
│                                                               │
│   Initializing personal operating environment...             │
│   [████████████████░░░░░░░░] 75%                             │
│                                                               │
│   ✓ Loading configuration                                    │
│   ✓ Starting tmux session                                    │
│   ⋯ Mounting workspace...                                    │
│                                                               │
│   System: MacBook Pro M1 • Battery: 87%                      │
└─────────────────────────────────────────────────────────────┘
```

### Boot Script

```bash
#!/usr/bin/env bash
# ~/bin/retro-os-boot

set -euo pipefail

# Check if session exists
if tmux has-session -t master 2>/dev/null; then
  # Session exists, attach
  tmux attach -t master
  exit 0
fi

# Show boot animation
clear
cat << 'EOF'
           ██████╗ ███████╗████████╗██████╗  ██████╗
           ██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██╔═══██╗
           ██████╔╝█████╗     ██║   ██████╔╝██║   ██║
           ██╔══██╗██╔══╝     ██║   ██╔══██╗██║   ██║
           ██║  ██║███████╗   ██║   ██║  ██║╚██████╔╝
           ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝
                          OS v1.0
EOF

echo ""
echo "Initializing personal operating environment..."
sleep 0.5

# Create session with 3-pane layout
tmux new-session -d -s master -n "retro-os"

# Split into 3 panes
tmux split-window -h -p 25  # Right panel (25%)
tmux select-pane -t 0
tmux split-window -h -p 20  # Left sidebar (15% of remaining 75%)

# Set pane titles
tmux select-pane -t 0 -T "Sidebar"
tmux select-pane -t 1 -T "Workspace"
tmux select-pane -t 2 -T "Info"

# Start components
tmux send-keys -t 0 "~/bin/retro-os-sidebar" C-m
tmux send-keys -t 1 "~/bin/retro-os-dashboard" C-m
tmux send-keys -t 2 "~/bin/retro-os-info-panel" C-m

# Focus workspace
tmux select-pane -t 1

# Attach
tmux attach -t master
```

---

## Navigation

### Keyboard Shortcuts

**Global**:
```
Ctrl+Space    Command palette (do anything)
Cmd+Space     Raycast (desktop apps)
Cmd+W         Dismiss current pod
Esc           Return to dashboard
Cmd+Q         Detach session (keeps running)
```

**Pane Navigation** (within RETRO-OS):
```
Ctrl+B H      Focus left (sidebar)
Ctrl+B L      Focus right (info panel)
Ctrl+B J/K    Cycle panes
```

**Quick Shortcuts** (in sidebar):
```
e             Email pod
f             Files pod
m             Music pod
t             Tasks pod
/             Search files
:             Command mode
```

**Pod-Specific**:
Each pod has its own keybindings (shown in pod UI)

---

## Retro Aesthetics

### Theme System

**Active Theme**: Catppuccin Mocha with CRT shader

**Switch Theme**: `Ctrl+Space` → `theme <name>`

**Available Themes**:
1. **Catppuccin Mocha** (default)
2. **Phosphor Green** (classic terminal green)
3. **Amber** (vintage monochrome)
4. **Commodore 64** (blue/purple retro)

### Visual Elements

**CRT Shader**: Enabled in Ghostty
```ini
custom-shader = ~/.config/ghostty/shaders/bettercrt.glsl
```

**Retro Fonts**:
- Inconsolata Nerd Font
- Monaspace Neon
- PragmataProMonoLiga

**ASCII Art**:
- Boot logo
- Dashboard borders
- Pod headers

**Sound Effects**:
```bash
# Boot complete
afplay /System/Library/Sounds/Glass.aiff

# Pod spawn
afplay /System/Library/Sounds/Ping.aiff

# Error
afplay /System/Library/Sounds/Basso.aiff
```

---

## Technical Implementation

### Directory Structure

```
~/dotfiles/
├── bin/
│   ├── retro-os-boot           # Boot script
│   ├── retro-os-sidebar        # Left sidebar
│   ├── retro-os-dashboard      # Main dashboard
│   ├── retro-os-info-panel     # Right panel
│   ├── retro-os-palette        # Command palette
│   └── pods/
│       ├── email.sh
│       ├── files.sh
│       ├── edit.sh
│       ├── music.sh
│       └── ...
├── config/
│   └── retro-os/
│       ├── config.yaml         # Main config
│       ├── pods.yaml           # Pod definitions
│       └── palette.yaml        # Command palette actions
```

### Command Palette Implementation

```bash
#!/usr/bin/env bash
# ~/bin/retro-os-palette

# Load pod definitions
PODS=(
  "📧 Email|~/bin/pods/email.sh"
  "📁 Files|~/bin/pods/files.sh"
  "✏️  Edit|~/bin/pods/edit.sh"
  "🎵 Music|~/bin/pods/music.sh"
  "📋 Tasks|~/bin/pods/tasks.sh"
  "📅 Calendar|~/bin/pods/calendar.sh"
  "🔧 Monitor|~/bin/pods/monitor.sh"
  "⚙️  Admin|~/bin/pods/admin.sh"
  "---"
  "✓ New Task|~/bin/pods/task-new.sh"
  "📝 New Note|~/bin/pods/note-new.sh"
  "🚀 Fresh Session|fresh"
  "---"
  "💾 Backup|dot backup create"
  "🔄 Update|dot update"
  "🩺 Health|dot health"
)

# fzf selection
selected=$(printf '%s\n' "${PODS[@]}" | \
  grep -v "^---$" | \
  fzf --prompt="RETRO-OS > " \
      --height=40% \
      --border=rounded \
      --preview-window=hidden \
      --bind 'ctrl-c:abort')

if [[ -n "$selected" ]]; then
  action=$(echo "$selected" | cut -d'|' -f2)

  # Execute in workspace pane (pane 1)
  tmux send-keys -t master:0.1 "$action" C-m
fi
```

### Pod Scripts

```bash
#!/usr/bin/env bash
# ~/bin/pods/email.sh

# Spawn email client in workspace
himalaya
```

```bash
#!/usr/bin/env bash
# ~/bin/pods/files.sh

# Spawn yazi as popup
tmux display-popup -E -w 80% -h 80% "yazi"
```

```bash
#!/usr/bin/env bash
# ~/bin/pods/music.sh

# Split workspace, spawn music at bottom
tmux split-window -v -p 30 "musikcube"
```

### Persistence

**tmux-resurrect config**:
```tmux
# Save/restore session
set -g @resurrect-save-interval '15'
set -g @resurrect-capture-pane-contents 'on'
set -g @resurrect-processes 'nvim vim htop yazi himalaya'

# Auto-restore on boot
run-shell '~/.config/tmux/plugins/tmux-resurrect/resurrect.tmux'
```

---

## Implementation Roadmap

### Phase 1: Core Layout (Week 1)

**Goals**:
- Create 3-pane layout
- Implement sidebar
- Implement dashboard
- Implement info panel

**Deliverables**:
- ✅ Boot script creates layout
- ✅ Sidebar shows actions/tasks
- ✅ Dashboard welcome screen
- ✅ Info panel shows calendar/system

**Tasks**:
- [ ] Write retro-os-boot
- [ ] Write retro-os-sidebar
- [ ] Write retro-os-dashboard
- [ ] Write retro-os-info-panel
- [ ] Test persistence

---

### Phase 2: Command Palette (Week 2)

**Goals**:
- Implement command palette
- Create pod launcher system
- Build 3-5 basic pods

**Deliverables**:
- ✅ Ctrl+Space opens palette
- ✅ Pods spawn/dismiss correctly
- ✅ Email, files, editor pods working

**Tasks**:
- [ ] Write retro-os-palette
- [ ] Create pod scripts (email, files, edit)
- [ ] Implement pod dismiss (Esc/Cmd+W)
- [ ] Test pod switching

---

### Phase 3: Pod Library (Week 3)

**Goals**:
- Build remaining pods
- Implement popup/split behaviors
- Quick capture system

**Deliverables**:
- ✅ All 10 pods working
- ✅ Popup pods (calendar, quick actions)
- ✅ Split pods (music, logs)
- ✅ Quick capture (task, note)

**Tasks**:
- [ ] Music pod (bottom split)
- [ ] Calendar pod (popup)
- [ ] Monitor pod
- [ ] Admin pod
- [ ] Quick captures

---

### Phase 4: Polish & Integration (Week 4)

**Goals**:
- Retro aesthetics
- Fresh integration
- PIM tools setup

**Deliverables**:
- ✅ CRT shader enabled
- ✅ Boot animation
- ✅ Fresh pod working
- ✅ Calendar/email syncing

**Tasks**:
- [ ] Enable CRT shader
- [ ] Boot animation
- [ ] Sound effects
- [ ] Configure himalaya (email)
- [ ] Configure calcurse (calendar)
- [ ] Fresh pod integration

---

### Phase 5: Launch (Week 5)

**Goals**:
- Auto-launch on login
- Documentation
- Demo/screenshots

**Deliverables**:
- ✅ LaunchAgent working
- ✅ User guide
- ✅ Demo video

**Tasks**:
- [ ] Create LaunchAgent
- [ ] Write user guide
- [ ] Record demo
- [ ] Blog post

---

## Quick Start

```bash
# Install
cd ~/dotfiles
git pull
./scripts/retro-os/install.sh

# Boot (or attach if running)
retro-os-boot

# Inside RETRO-OS:
Ctrl+Space          # Command palette
Type "email"        # Open email
Type "files"        # Browse files
Type "task"         # New task
Esc                 # Return to dashboard
Cmd+W               # Dismiss pod
Cmd+Q               # Detach (keeps running)
```

---

## Conclusion

RETRO-OS reimagines your terminal as a complete operating environment:

✨ **One Session**: Permanent, never dies
🎯 **Command Palette**: Ctrl+Space for everything
🎨 **Retro Aesthetic**: CRT shaders, retro fonts
⚡ **Modern Power**: AI, fuzzy search, cloud sync
📦 **Pod System**: Tools on-demand, not pre-allocated
🏠 **Persistent**: Survives reboots, exact state

This isn't a tmux config—it's your personal computing environment.

Welcome home. 🎮

---

**Version**: 2.0
**Created**: 2025-10-16
**Author**: Built with Claude Code
**License**: MIT (part of dotfiles)
