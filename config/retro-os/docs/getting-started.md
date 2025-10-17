# RETRO-OS: Getting Started

Welcome to your personal operating environment!

## Launch

```bash
retro-os        # Boot or attach to RETRO-OS
```

## Core Concept

RETRO-OS is a permanent tmux session with three panes:
- **Left Sidebar (15%)**: Quick actions, recent projects, tasks
- **Center Workspace (60%)**: Dashboard or active pod
- **Right Info Panel (25%)**: Calendar, system stats

## Command Palette

Press **Ctrl+Space** anywhere to open the command palette.

Type to filter, Enter to select.

## Pods

Pods are tools that spawn on-demand:
- Files, Editor, Monitor, Music, Tasks, Notes, Git
- Email, Calendar (mocked, real coming in Phase 6)
- Admin, Help

Press **Esc** or **Cmd+W** to dismiss a pod and return to dashboard.

## Quick Captures

- **New Task**: Ctrl+Space → "New Task" → type task
- **New Note**: Ctrl+Space → "New Note" → write note

## Navigation

- **Cmd+0**: Return to RETRO-OS (window 0)
- **Cmd+1-9**: Switch to other tmux windows (fresh sessions)
- **Alt+H/J/K/L**: Switch panes

## Fresh Sessions

RETRO-OS coexists with your `fresh` project sessions:
- Window 0: RETRO-OS (permanent)
- Windows 1+: Fresh project sessions

Launch fresh session: `fresh` or via sesh
