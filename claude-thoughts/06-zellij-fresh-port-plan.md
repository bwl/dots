# Porting Fresh to Zellij

## Current Fresh (Tmux) Design

### What It Does
- Creates project-specific session named after current directory
- 5 windows: claude, codex, cliffy, git, tasks
- Windows 0-3 have 70/30 split with taskbook sidebar
- Window 4 is full-screen taskbook
- Auto-refreshing taskbook display (2s interval)

### Commands
- `fresh` - Create/attach to session
- `fresh explain` - Show context for agents
- `fresh kill` - Kill current session

## Proposed Zellij Version

### Architecture Changes

**Tabs instead of Windows**
- Tab 0: claude
- Tab 1: codex
- Tab 2: cliffy
- Tab 3: git
- No dedicated tasks tab

**Taskbook as Floating Pane**
- Toggleable with Ctrl-T
- Appears as overlay (50% width, 80% height, centered)
- Auto-refreshing taskbook command
- Available from any tab

### Benefits of This Approach
1. **Less clutter** - Tasks only visible when needed
2. **More screen space** - Full width for agent windows
3. **Better UX** - Quick toggle vs switching to dedicated tab
4. **Modern design** - Floating panes are Zellij's strength

### Implementation

#### 1. Layout File: `~/.config/zellij/layouts/fresh.kdl`

```kdl
layout {
    // Set the default CWD for all panes
    cwd "$PROJECT_DIR"

    // Tab 0: Claude
    tab name="claude" {
        pane command="claude"
    }

    // Tab 1: Codex
    tab name="codex" {
        pane command="codex"
    }

    // Tab 2: Cliffy
    tab name="cliffy" {
        pane command="cliffy"
    }

    // Tab 3: Git (lazygit)
    tab name="git" {
        pane command="lazygit"
    }

    // Floating taskbook pane (toggled with Ctrl-T)
    floating_panes {
        pane name="tasks" {
            command "bash"
            args "-c" "source ~/.zshrc && while true; do tb_cmd='tb'; [[ -d .taskbook ]] && tb_cmd='tb --storage-dir ./.taskbook'; clear; $tb_cmd; sleep 2; done"
            x "25%"
            y "10%"
            width "50%"
            height "80%"
        }
    }
}
```

#### 2. Keybinding Config: `~/.config/zellij/config.kdl`

```kdl
keybinds {
    normal {
        // Toggle taskbook floating pane with Ctrl-T
        bind "Ctrl t" { ToggleFloatingPanes; }

        // Quick tab switching with Alt-1 through Alt-4
        bind "Alt 1" { GoToTab 1; }
        bind "Alt 2" { GoToTab 2; }
        bind "Alt 3" { GoToTab 3; }
        bind "Alt 4" { GoToTab 4; }
    }
}
```

#### 3. Fresh Script: `~/dotfiles/bin/fresh-zellij`

```bash
#!/usr/bin/env bash
set -euo pipefail

if ! command -v zellij >/dev/null 2>&1; then
  echo "zellij is not installed" >&2
  exit 1
fi

# Handle subcommands
case "${1:-}" in
  explain)
    if [[ -z "${ZELLIJ:-}" ]]; then
      echo "Not in a Zellij session"
      exit 0
    fi

    # Show Zellij context
    echo "=== Zellij Context ==="
    echo "Session: $(zellij list-sessions | grep EXITED -v | head -1 | awk '{print $1}')"
    echo ""
    echo "=== Agent Profiles ==="
    cat ~/dotfiles/scripts/tmux-fresh/agent_profiles.json | jq -r '
      to_entries[] |
      "Tab: \(.key)\n  Name: \(.value.name)\n  Role: \(.value.role)\n  Description: \(.value.description)\n"
    '
    exit 0
    ;;

  kill)
    if [[ -z "${ZELLIJ:-}" ]]; then
      echo "Not in a Zellij session" >&2
      exit 1
    fi
    zellij kill-session
    exit 0
    ;;
esac

# Get project directory and session name
PROJECT_DIR=$(pwd)
SESSION_NAME=$(basename "$PROJECT_DIR" | sed 's/[^a-zA-Z0-9_-]/_/g')

# Check if session exists
if zellij list-sessions 2>/dev/null | grep -q "^$SESSION_NAME"; then
  # Attach to existing session
  exec zellij attach "$SESSION_NAME"
else
  # Create new session with fresh layout
  export PROJECT_DIR  # Make available to layout
  exec zellij --session "$SESSION_NAME" --layout fresh
fi
```

### Migration Path

**Option A: Side-by-side**
- Keep `fresh` (tmux version)
- Create `fresh-zellij` (new version)
- Test Zellij for a week
- Choose one to keep as `fresh`

**Option B: Replace immediately**
- Rename `fresh` → `fresh-tmux`
- Create `fresh` as Zellij version
- Use Zellij as primary
- Fallback to tmux if needed

## Questions to Answer

1. **Can Zellij source .zshrc in panes?**
   - Needed for `claude`, `codex`, `cliffy` aliases
   - May need to wrap commands in bash/zsh -c

2. **Does floating pane auto-refresh work?**
   - Need to test `while true; do tb; sleep 2; done` in floating pane
   - May need different approach

3. **Can we pass $PROJECT_DIR to layout?**
   - Zellij supports environment variables in layouts
   - Need to test if `cwd "$PROJECT_DIR"` works

4. **Is Ctrl-T a good binding?**
   - Doesn't conflict with Zellij defaults
   - Easy to reach
   - Could also use Alt-T or Alt-\`

## Testing Checklist

- [ ] Create layout file
- [ ] Test layout manually: `zellij --layout ~/.config/zellij/layouts/fresh.kdl`
- [ ] Verify all commands launch correctly
- [ ] Test floating pane toggle
- [ ] Test taskbook auto-refresh in floating pane
- [ ] Verify project directory is correct in all panes
- [ ] Create keybinding config
- [ ] Test Ctrl-T binding
- [ ] Create fresh-zellij script
- [ ] Test session creation
- [ ] Test session reattachment
- [ ] Test `explain` and `kill` subcommands
- [ ] Compare UX to tmux version

## Success Criteria

Zellij version is successful if:
1. All agent commands launch correctly
2. Taskbook toggles smoothly with Ctrl-T
3. Session management works (create/attach/kill)
4. Project directory is respected
5. UX feels better than tmux version
6. No significant bugs or limitations

If any of these fail, we stick with tmux.
