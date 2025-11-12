# Configuration Organization: Structure for Clarity

## Current Structure Analysis

```
dotfiles/
  config/
    tmux/
      tmux.conf                    # Main config
      plugins/                      # All plugins as submodules
        catppuccin/
        tmux-battery/
        tmux-cpu/
        tmux-fzf/
        tmux-harpoon/
        ... (many more)
```

Symlinked to:
```
~/.tmux.conf -> ~/dotfiles/config/tmux/tmux.conf
~/.tmux/plugins/* -> (via plugin manager)
```

### Problems with This Structure

1. **Dual ownership**: `plugins/` contains code you don't own
2. **Unclear boundaries**: Which code can you modify?
3. **Update confusion**: Submodules need updating, but breaks your changes
4. **Navigation complexity**: Is code in the plugin or your config?
5. **Testing difficulty**: Can't test without all submodules present

## Proposed Structure: Own Everything

```
dotfiles/
  config/
    tmux/
      tmux.conf                    # Main config
      scripts/                     # YOUR scripts
        status-right.sh            # Status bar generator
        cpu.sh                     # CPU monitoring (if needed separately)
        battery.sh                 # Battery monitoring (if needed separately)
        session-menu.sh            # Custom session switcher
      colors/                      # YOUR color definitions
        catppuccin-mocha.conf      # Vendored theme (if you want)
        custom.conf                # Your overrides
      keybindings.conf            # Separate file for keys
      appearance.conf             # Separate file for look/feel
      README.md                   # YOUR documentation of your setup
```

Symlinked to:
```
~/.tmux.conf -> ~/dotfiles/config/tmux/tmux.conf
```

That's it. No complex plugin directory structure.

### Principles

1. **Everything is yours**: If it's in your dotfiles, you own it
2. **Clear purpose**: Directory names describe content
3. **Flat where possible**: Don't nest unless there's clear benefit
4. **Self-documenting**: Structure reveals intent

## The Config File Strategy

### Option 1: Monolithic tmux.conf

One file with everything:
```tmux
# ~/.tmux.conf
# Mouse, history, colors, keys, status bar, everything
```

**Pros**:
- Everything in one place
- No hunting across files
- Easy to understand flow

**Cons**:
- Can get long
- Mixing concerns (keys + colors + behavior)

**Good for**: Configs under ~200 lines

### Option 2: Modular tmux.conf

Main file sources components:
```tmux
# ~/.tmux.conf
source-file ~/.tmux/keybindings.conf
source-file ~/.tmux/appearance.conf
source-file ~/.tmux/status-bar.conf
source-file ~/.tmux/behavior.conf
```

**Pros**:
- Organized by concern
- Easy to find specific settings
- Can disable sections by commenting one line

**Cons**:
- More files to track
- Have to remember which file has what
- Order dependencies (colors before status bar, etc.)

**Good for**: Configs over ~200 lines, or configs shared across machines

### Option 3: Hybrid

Main config has core settings, sources extras:
```tmux
# ~/.tmux.conf
# Core settings (mouse, history, escape-time, etc.)
set -g mouse on
set -g history-limit 1000000
set -sg escape-time 0

# Appearance (colors, status bar)
source-file ~/.tmux/appearance.conf

# Keybindings
source-file ~/.tmux/keybindings.conf
```

**Pros**:
- Best of both worlds
- Core is always visible
- Extensions are modular

**Cons**:
- Have to decide what goes where

**Good for**: Most users (this is what I'd recommend)

## The Script Organization Question

For tmux scripts, you have options:

### Option A: Single Status Script

```bash
# ~/.tmux/scripts/status-right.sh
# Does everything: CPU, battery, time, etc.
```

Used as:
```tmux
set -g status-right "#(~/.tmux/scripts/status-right.sh)"
```

**Pros**:
- One script to maintain
- Easy to coordinate between elements (spacing, colors)
- Single execution (faster)

**Cons**:
- Mixing concerns
- Harder to reuse parts

### Option B: Separate Scripts Per Component

```bash
~/.tmux/scripts/cpu.sh
~/.tmux/scripts/battery.sh
~/.tmux/scripts/datetime.sh
```

Used as:
```tmux
set -g status-right "#(~/.tmux/scripts/cpu.sh) #(~/.tmux/scripts/battery.sh) #(~/.tmux/scripts/datetime.sh)"
```

**Pros**:
- Modular
- Can test each part separately
- Easy to add/remove components

**Cons**:
- Multiple processes (slightly slower)
- Coordinating colors/spacing is harder
- More files

### Option C: Hybrid (Recommended)

```bash
# ~/.tmux/scripts/status-right.sh (main script)
# Sources helper functions from lib/

source "$(dirname "$0")/lib/system-info.sh"

cpu=$(get_cpu_percentage)
battery=$(get_battery_percentage)
# ... format and display
```

```bash
# ~/.tmux/scripts/lib/system-info.sh
get_cpu_percentage() {
    top -l 2 -n 0 -F -s 0 | grep "CPU usage" | tail -1 | awk '{print $3}'
}

get_battery_percentage() {
    pmset -g batt | grep -Eo "\d+%" | cut -d% -f1
}
```

**Pros**:
- Single execution (fast)
- Modular code (testable)
- Clean separation of data gathering vs display

**Cons**:
- More initial setup

## Where Should Color Definitions Live?

### Current: In Plugin

Catppuccin plugin defines:
```tmux
thm_yellow="#f9e2af"
thm_blue="#b4befe"
# ... dozens more
```

### Option 1: In tmux.conf

```tmux
# ~/.tmux.conf
%hidden color_accent="#f9e2af"
%hidden color_highlight="#b4befe"
%hidden color_bg="#313244"
```

**Pros**: Colors visible next to usage
**Cons**: Clutters main config

### Option 2: Separate Colors File

```tmux
# ~/.tmux/colors.conf
# Catppuccin Mocha palette
%hidden thm_yellow="#f9e2af"
# ...
```

```tmux
# ~/.tmux.conf
source-file ~/.tmux/colors.conf
set -g status-right "#[fg=$thm_yellow]..."
```

**Pros**: Clean separation, easy to swap themes
**Cons**: Another file

### Option 3: In Scripts

```bash
# ~/.tmux/scripts/status-right.sh
YELLOW="#f9e2af"
BLUE="#b4befe"

echo "#[fg=$YELLOW] CPU: ... #[fg=$BLUE] Battery: ..."
```

**Pros**: Colors next to usage, self-contained script
**Cons**: Harder to coordinate with other tmux colors

### Recommendation

For your single-laptop, single-terminal use case:

**Put colors in a separate file if you want themability**, or **hardcode them in scripts if you want simplicity**.

You're not running a tmux rice showcase. You want it to look good and work well. Pick a few colors you like, use them consistently, move on.

## Documentation Strategy

Every dotfiles repo needs documentation, but there's a trap: **over-documenting configuration.**

### Don't Document: What the Code Does

Bad:
```tmux
# Set mouse on
set -g mouse on
```

The code is self-explanatory. The comment adds no value.

### Do Document: Why You Made Choices

Good:
```tmux
# Mouse on for scrollback, but use keyboard for selections (faster)
set -g mouse on
bind -n WheelUpPane select-pane -t= \; copy-mode -e \; send-keys -M
```

Or:
```tmux
# Status refresh every 2s - balance between current info and CPU usage
set -g status-interval 2
```

### The README.md

Your `~/.tmux/README.md` should answer:
1. **What's my setup philosophy?** (minimal, fast, maintainable)
2. **What are the keybindings I actually use?** (not all of them, the ones you rely on)
3. **How do I modify common things?** (add status item, change colors, add keybind)
4. **What did I vendor and why?** (if anything)

It should NOT be:
- A tutorial on tmux (that's what `man tmux` is for)
- A complete reference (that's what the config file itself is for)
- A changelog of every tweak

## The Test: Can You Rebuild It?

Good organization means: if your laptop dies, can you rebuild this setup quickly?

With plugins/submodules:
1. Clone dotfiles
2. Run symlink script
3. Run `git submodule init && git submodule update`
4. Run TPM installer
5. Hope everything works
6. Debug when it doesn't

With owned code:
1. Clone dotfiles
2. Run symlink script
3. Done

The second approach is testable: you can literally test it in a fresh VM/container.

## Proposed Structure for Your Dotfiles

```
dotfiles/
  config/
    tmux/
      tmux.conf                 # Main config (~100 lines)
      appearance.conf           # Colors, status bar, window styling (~50 lines)
      scripts/
        status-right.sh         # Your status bar (~30 lines)
        lib/                    # Shared functions if needed
          system-info.sh
      README.md                 # Your docs
```

Total complexity: ~200 lines of config, ~50 lines of script.
All yours. All understandable. All maintainable with Claude.

Compare to current:
- 10+ plugin submodules
- TPM infrastructure
- Catppuccin wrapper system
- 100s of lines of code you don't own

Which would you rather maintain?
