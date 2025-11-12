# Migration Path: From Plugin Hell to Owned Simplicity

## Current State Assessment

You have:
- 10+ tmux plugins as git submodules
- TPM managing plugin loading
- Catppuccin as a wrapper around other plugins
- Complex load ordering issues (why CPU doesn't work)
- Mental overhead of "which plugin does what"

You want:
- Simple, maintainable config
- Full ownership of code
- Ability to modify anything with Claude's help
- No dependency juggling

## Migration Philosophy

**Don't big-bang rewrite.** That's how you lose a working setup.

Instead: **Progressive ownership.**
- Extract one plugin at a time
- Keep things working throughout
- Build confidence in the new approach
- Validate each step

## Phase 1: Audit (Do This First)

### Step 1: Document Current Functionality

What does your tmux actually DO that you care about?

Create a checklist:
```
Status bar:
- [ ] Shows CPU percentage
- [ ] Shows battery level
- [ ] Shows date/time
- [ ] Shows uptime
- [ ] Uses Catppuccin colors

Keybindings:
- [ ] Alt-arrow for pane navigation
- [ ] Shift-arrow for window navigation
- [ ] Alt-1..9 for window jumping
- [ ] (list others you actually use)

Features:
- [ ] Mouse support for scrollback
- [ ] Sidebar (do you use this?)
- [ ] Harpoon (do you use this?)
- [ ] FZF picker (do you use this?)
- [ ] (list others)
```

### Step 2: Test What Actually Works

In your current setup:
- Which features work?
- Which are broken? (CPU is broken)
- Which do you never use?

Be honest. When's the last time you used tmux-sidebar?

### Step 3: Screenshot Your Current Look

You like how it looks (minus the broken CPU). Capture that:
```bash
# Take a screenshot of your status bar
# This is your "design spec" for the rebuild
```

## Phase 2: Create Parallel Config (Safe Testing)

Don't modify your working tmux.conf yet. Create a test version:

```bash
cp ~/.tmux.conf ~/.tmux-new.conf
```

Test it in a new session:
```bash
tmux -f ~/.tmux-new.conf new-session -s test
```

This lets you experiment without breaking your main setup.

## Phase 3: Extract One Plugin At A Time

### Round 1: The Easiest Win - CPU/Battery

These are broken or overly complex anyway.

#### Current (in plugins):
```
~/.tmux/plugins/tmux-cpu/*
~/.tmux/plugins/tmux-battery/*
```

#### New (owned):
```bash
# Create the script
mkdir -p ~/.tmux/scripts
cat > ~/.tmux/scripts/status-right.sh << 'EOF'
#!/bin/bash
cpu=$(top -l 2 -n 0 -F -s 0 | grep "CPU usage" | tail -1 | awk '{print $3}')
battery=$(pmset -g batt | grep -Eo "\d+%" | cut -d% -f1)
date=$(date "+%H:%M")
echo " $cpu  $battery%  $date"
EOF
chmod +x ~/.tmux/scripts/status-right.sh
```

#### Update config:
```tmux
# In ~/.tmux-new.conf
# Replace the complex catppuccin status-right with:
set -g status-right "#(~/.tmux/scripts/status-right.sh)"
```

#### Test:
```bash
tmux -f ~/.tmux-new.conf new-session -s test
# Does the status bar work?
# Does it show actual percentages?
```

#### Iterate:
```bash
# Add colors back if you want:
cat > ~/.tmux/scripts/status-right.sh << 'EOF'
#!/bin/bash
YELLOW="#f9e2af"
BLUE="#b4befe"
FG="#cdd6f4"

cpu=$(top -l 2 -n 0 -F -s 0 | grep "CPU usage" | tail -1 | awk '{print $3}')
battery=$(pmset -g batt | grep -Eo "\d+%" | cut -d% -f1)
date=$(date "+%H:%M")

echo "#[fg=$YELLOW] $cpu #[fg=$BLUE] $battery% #[fg=$FG] $date"
EOF
```

#### Validate:
- [ ] CPU shows actual percentage (not #{cpu_percentage})
- [ ] Battery shows percentage
- [ ] Colors look good
- [ ] Updates regularly (check `set -g status-interval 2`)

Once working: **Remove tmux-cpu and tmux-battery from your config.**

### Round 2: tmux-sensible (Config Only)

This plugin just sets sane defaults. Copy them directly.

#### Current:
```tmux
set -g @plugin 'tmux-plugins/tmux-sensible'
```

#### New:
```tmux
# From tmux-sensible, settings I want:
set -sg escape-time 0
set -g history-limit 50000
set -g display-time 4000
set -g status-interval 5
set -g focus-events on
setw -g aggressive-resize on
```

#### Test:
- [ ] Escape key responsive
- [ ] Scrollback works
- [ ] Display messages show long enough

Once working: **Remove tmux-sensible plugin.**

### Round 3: Catppuccin (Colors Only)

You like the colors. Keep the colors. Lose the infrastructure.

#### Current:
```tmux
set -g @plugin 'catppuccin/tmux'
# Plus complex status bar configuration
```

#### New:
Extract just the colors:
```tmux
# Catppuccin Mocha - colors only
%hidden thm_bg="#1e1e2e"
%hidden thm_fg="#cdd6f4"
%hidden thm_cyan="#89dceb"
%hidden thm_green="#a6e3a1"
%hidden thm_yellow="#f9e2af"
%hidden thm_blue="#b4befe"
%hidden thm_magenta="#cba6f7"
%hidden thm_red="#f38ba8"

# Use them in your config:
set -g status-style "bg=$thm_bg,fg=$thm_fg"
set -g window-status-current-style "fg=$thm_yellow"
# etc.
```

#### Test:
- [ ] Colors look the same
- [ ] Status bar styled correctly
- [ ] Window names styled correctly

Once working: **Remove catppuccin plugin.**

### Round 4-N: Assess Remaining Plugins

For each remaining plugin, ask:

**1. Do I use this?**
- If no: Remove immediately.
- If unsure: Remove and see if you notice.

**2. What does it actually do?**
- Read the README
- Check what keybindings/commands it adds
- Test if you actually invoke those

**3. Is it trivial to replace?**
- If <50 lines of logic: Replace
- If config only: Copy config, remove plugin
- If complex: Decide if you really need it

**4. Can I live without it temporarily?**
- If yes: Remove, replace later if missed
- If no: Vendor it properly (full copy, not submodule)

## Phase 4: Remove TPM

Once you have zero plugins left:

```tmux
# Remove from ~/.tmux-new.conf:
set -g @plugin ...  # All of these
run -b '~/.tmux/plugins/tpm/tpm'  # This line
```

You might have:
```bash
# Optional helper scripts
run-shell ~/.tmux/scripts/some-custom-thing.sh
```

But no plugin manager, no @plugin directives.

## Phase 5: Remove Submodules

This is the scary part, but you've already replaced the functionality.

### Backup First
```bash
cd ~/dotfiles
git branch backup-before-plugin-removal
```

### Remove Each Submodule
```bash
# For each plugin:
git submodule deinit config/tmux/plugins/tmux-cpu
git rm config/tmux/plugins/tmux-cpu
rm -rf .git/modules/config/tmux/plugins/tmux-cpu

# Repeat for all plugins
```

### Or Nuke From Orbit
```bash
git submodule deinit config/tmux/plugins
git rm config/tmux/plugins
rm -rf .git/modules/config/tmux/plugins
```

### Update .gitmodules
Should now be empty (or have only non-tmux submodules if any).

### Commit
```bash
git add .
git commit -m "Remove tmux plugin infrastructure, replace with owned implementations"
```

## Phase 6: Reorganize

Now that you own everything, make it clean:

### Before:
```
config/tmux/
  tmux.conf
  plugins/
    (deleted submodules)
```

### After:
```
config/tmux/
  tmux.conf           # Main config
  scripts/            # Your scripts
    status-right.sh
  README.md           # Document your setup
```

Simple. Clean. Yours.

## Phase 7: Documentation

Create `config/tmux/README.md`:

```markdown
# My Tmux Configuration

## Philosophy
- Own all code
- Simple, maintainable scripts
- AI-assisted customization
- No external plugins

## Structure
- `tmux.conf`: Main configuration
- `scripts/`: Shell scripts for status bar, etc.

## Status Bar
Custom script showing: CPU, battery, time.
Colors from Catppuccin Mocha theme.

## Maintenance
To modify: edit the relevant file, ask Claude for help.
No plugin manager, no submodules, no magic.

## Attribution
- Color scheme inspired by Catppuccin Mocha
- Original plugin references (for history):
  - tmux-cpu → now `scripts/status-right.sh`
  - tmux-battery → now `scripts/status-right.sh`
  - tmux-sensible → settings in `tmux.conf`
```

## Phase 8: Validation

Before making the new config your default:

### Checklist
- [ ] All features you care about work
- [ ] Looks the same (or better)
- [ ] No broken references to plugins
- [ ] Scripts are executable
- [ ] Test in fresh tmux session
- [ ] Test after reboot
- [ ] Test sourcing config with `prefix + r`

### Make It Default
```bash
# Backup old config
mv ~/.tmux.conf ~/.tmux.conf.old

# Symlink to new config (already done if you edited in dotfiles)
# Just update which file you're using in your symlink script

# Or just:
mv ~/.tmux-new.conf ~/.tmux.conf
```

### New Tmux Session
```bash
tmux new-session
# Everything works?
```

## Phase 9: Update Dotfiles Docs

Update your main CLAUDE.md:

### Remove:
- References to TPM
- Instructions for plugin installation
- Submodule update commands

### Add:
- Philosophy of owned code
- How to customize scripts
- How to work with Claude on tmux config

## Rollback Plan

If anything goes wrong:

```bash
# You have backups:
git checkout backup-before-plugin-removal

# Or:
cp ~/.tmux.conf.old ~/.tmux.conf

# Restart tmux:
tmux kill-server
tmux
```

## Timeline

This doesn't have to be done all at once:

**Week 1**: Audit, create test config, fix CPU/battery
**Week 2**: Remove simple plugins (sensible, etc.)
**Week 3**: Simplify colors, remove Catppuccin infrastructure
**Week 4**: Assess remaining plugins, remove TPM
**Week 5**: Clean up submodules, reorganize, document

Or do it all in an afternoon if you're feeling bold.

## Success Criteria

You'll know you're done when:

1. ✅ Your tmux works exactly as you want
2. ✅ You can explain what every line of config does
3. ✅ You can modify any behavior by editing one file
4. ✅ No git submodules in `~/.tmux/`
5. ✅ No plugin manager
6. ✅ Claude can help you customize anything instantly

## The After State

```
config/tmux/
  tmux.conf          # ~150 lines, all yours
  scripts/
    status-right.sh  # ~20 lines, all yours
  README.md          # Your docs
```

Zero submodules. Zero external dependencies (except tmux itself).
100% ownership. 100% understanding.

That's the goal.
