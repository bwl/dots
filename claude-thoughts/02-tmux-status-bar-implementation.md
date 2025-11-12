# Tmux Status Bar: Implementation Options Analysis

## Current Situation

Your status bar shows:
- CPU percentage (broken - shows literal `#{cpu_percentage}`)
- Battery icon + percentage (working)
- Date/time
- Uptime

It's using:
- Catppuccin theme (wrapper around other plugins)
- tmux-cpu plugin (provides CPU data)
- tmux-battery plugin (provides battery data)
- Various TPM infrastructure

## The Layers of Abstraction

Current stack (bottom to top):
1. **Shell scripts** (`cpu_percentage.sh`, `battery_percentage.sh`) - do the actual work
2. **Plugin .tmux files** - register variables, do tmux interpolation
3. **Catppuccin wrappers** - wrap plugin variables with theme colors
4. **TPM** - manages installation and loading
5. **Your tmux.conf** - orchestrates all of this

That's 5 layers to show a percentage on screen.

## Option 1: Direct Implementation (Simplest)

### What You Actually Need

```bash
# ~/.tmux/scripts/cpu.sh
#!/bin/bash
top -l 2 -n 0 -F -s 0 | grep "CPU usage" | tail -1 | awk '{print $3}' | sed 's/%//'
```

```bash
# ~/.tmux/scripts/battery.sh
#!/bin/bash
pmset -g batt | grep -Eo "\d+%" | cut -d% -f1
```

### Tmux Config
```tmux
set -g status-right "#(~/.tmux/scripts/cpu.sh)%  #(~/.tmux/scripts/battery.sh)%  %H:%M"
```

**That's it.** No plugins, no managers, no interpolation, no wrappers.

### Pros
- Transparent: you can read exactly what happens
- Fast: no indirection layers
- Maintainable: ~5 lines of shell script
- Debuggable: run the script directly to test
- Customizable: change the format instantly

### Cons
- No pre-made theming
- Need to write your own color codes
- Less modular (but do you need modularity for 5 lines?)

## Option 2: Vendored Plugin Code (Moderate)

Keep the plugin structure but own it:

```
~/.tmux/
  scripts/
    cpu/
      cpu.sh         # from tmux-cpu plugin
      helpers.sh     # any dependencies
    battery/
      battery.sh     # from tmux-battery plugin
```

Reference in tmux.conf:
```tmux
set -g status-right "#(~/.tmux/scripts/cpu/cpu.sh) #(~/.tmux/scripts/battery/battery.sh)"
```

### Pros
- Start with battle-tested code
- Can simplify over time (remove unused OS support, etc.)
- Own the code but leverage prior art
- No external dependencies

### Cons
- More code than you need
- Still maintaining someone else's design
- Harder to understand than writing from scratch

## Option 3: Catppuccin Direct (Theme-Focused)

If you love Catppuccin colors and want to keep them:

Vendor the catppuccin theme but make it standalone:
- Copy the color definitions you actually use
- Write status bar config directly with those colors
- Skip the plugin module system

```tmux
# Colors (from Catppuccin Mocha)
%hidden thm_yellow="#f9e2af"
%hidden thm_blue="#b4befe"
%hidden thm_bg="#313244"
%hidden thm_fg="#cdd6f4"

# Status bar with theme colors
set -g status-right "#[fg=$thm_yellow] #(~/.tmux/scripts/cpu.sh)% #[fg=$thm_blue] #(~/.tmux/scripts/battery.sh)%"
```

### Pros
- Keep the aesthetics you like
- Remove the plugin infrastructure
- Colors are just variables, not a "plugin"

### Cons
- Still more complex than raw
- Color variables add indirection

## Option 4: Hybrid - Smart Vendoring

The pragmatic middle ground:

1. **Own simple code**: CPU, battery, date are trivial - write fresh
2. **Vendor complex code**: If you need something genuinely complex, bring it in as reference
3. **Graduate over time**: Start with vendored code, simplify as you understand it

Example progression:
- Week 1: Copy `tmux-cpu` scripts wholesale, they work
- Week 2: "Claude, remove Windows/Linux support, I only use macOS"
- Week 3: "Claude, I don't need GPU monitoring, remove it"
- Week 4: You now have a 20-line script you fully understand

## The Real Question: What Do You Actually Want?

Let's be specific. Your status bar currently tries to show:
1. CPU percentage
2. Battery icon + percentage
3. Date/time
4. Uptime

Do you actually want all of these? Let's think about each:

### CPU Percentage
- **Use case**: Knowing if something is spinning
- **Frequency**: Glance occasionally
- **Complexity**: One `top` command
- **Verdict**: Trivial, write fresh

### Battery
- **Use case**: Knowing when to plug in
- **Frequency**: Important on laptop
- **Complexity**: One `pmset` command
- **Verdict**: Trivial, write fresh

### Date/Time
- **Use case**: Current time
- **Frequency**: Constant reference
- **Complexity**: tmux built-in `%H:%M`
- **Verdict**: Already trivial, no script needed

### Uptime
- **Use case**: ... do you actually check this?
- **Frequency**: Rarely?
- **Complexity**: One `uptime` command
- **Verdict**: Questionable value, consider removing

### What About Icons/Colors?

The Catppuccin theme adds:
- Color-coded backgrounds
- Emoji/icon prefixes
- Separators and shapes

**Ask yourself**: Is this visual candy worth 3 layers of indirection?

Alternative: Pick 2-3 colors you like, hardcode them. Done.

## Recommendation for Your Setup

Based on your stated preferences (focus on UX, one laptop, one terminal):

### The Minimalist Status Bar

```bash
# ~/.tmux/scripts/status-right.sh
#!/bin/bash

# Simple, readable, maintained by you + Claude
cpu=$(top -l 2 -n 0 -F -s 0 | grep "CPU usage" | tail -1 | awk '{print $3}')
battery=$(pmset -g batt | grep -Eo "\d+%" | cut -d% -f1)
date=$(date "+%Y-%m-%d %H:%M")

echo " $cpu  $battery%  $date"
```

```tmux
# ~/.tmux.conf
set -g status-right "#(~/.tmux/scripts/status-right.sh)"
set -g status-interval 2
```

**Total complexity**: 1 script file, 1 config line.

**Customization**: Change the script. That's it. No digging through plugin docs.

**Debugging**: Run `~/.tmux/scripts/status-right.sh` and see the output.

**AI Maintenance**: "Claude, add memory usage" - modifies one file.

### If You Want Colors

```bash
# Add color codes to the script output
echo "#[fg=#f9e2af] $cpu #[fg=#b4befe] $battery% #[fg=#cdd6f4] $date"
```

Ugly? Yes. But transparent. You can see exactly what displays.

Want prettier? Write a helper:
```bash
colored() {
    local color=$1
    local text=$2
    echo "#[fg=$color]$text"
}

echo "$(colored '#f9e2af' " $cpu") $(colored '#b4befe' " $battery%") ..."
```

Still one script. Still yours.

## The Laptop + One Terminal UX Consideration

You mentioned "laptop with one terminal window" - this is key.

You're not running tmux on a server with 50 windows. You're using it as a local multiplexer for a few specific workflows (claude, codex, cliffy, git, tasks based on your fresh script).

**What does your status bar actually need to do?**

Option A: Just show you time and battery (the essentials for laptop work)
```
2025-10-17 18:53  87%
```

Option B: Show context for your AI workspace
```
claude  ~/dotfiles  18:53  87%
```
(window name, current dir, time, battery)

Option C: Full system monitor mode
```
CPU: 35%  RAM: 64%  SSD: 45%  Battery: 87%  2025-10-17 18:53  Uptime: 7h 59m
```

Which aligns with your actual usage patterns?

My guess: Option A or B. You don't need extensive system monitoring - you have dedicated tools for that when needed.

## Conclusion

For tmux status bar in 2025:

**Write. Your. Own. Script.**

It's ~10 lines of bash.
It does exactly what you need.
You can modify it in 30 seconds.
Claude can help you with any changes.

Stop maintaining dependency graphs for functionality you could implement in an afternoon.

The plugin ecosystem made sense when implementation was expensive.
Implementation is no longer expensive.
