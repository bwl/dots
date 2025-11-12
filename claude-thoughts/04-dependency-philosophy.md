# Dependency Philosophy: What Should Actually Be External?

## The Dependency Spectrum

Not all external code is created equal. There's a spectrum:

```
[Own It] ←────────────────────────────────────────→ [Depend On It]
  Config       Scripts      Libraries      Tools        OS/Runtime
   │              │            │             │              │
 tmux.conf   status.sh    (none here)     tmux          bash/zsh
```

Let's analyze where tmux "plugins" fall on this spectrum.

## Category 1: Pure Config (Own It)

**What it is**: Configuration snippets, keybindings, color schemes

**Examples from your setup**:
- Catppuccin theme colors
- Window styling
- Status bar format
- Keybinding definitions

**Why own it**:
- It's literally YOUR configuration
- No complex logic, just settings
- Changes constantly based on preferences
- No code to "maintain", just values to adjust

**How to vendor**:
- Copy the values you like
- Remove everything else
- Put in your config file or a sourced file
- Done

**Example**: Catppuccin colors
```tmux
# Instead of: git submodule for catppuccin plugin
# Just do:
%hidden thm_bg="#1e1e2e"
%hidden thm_fg="#cdd6f4"
%hidden thm_yellow="#f9e2af"
# ... the 5-10 colors you actually use
```

## Category 2: Simple Scripts (Own It)

**What it is**: Shell scripts <100 lines that do one thing

**Examples from your setup**:
- CPU percentage display (tmux-cpu)
- Battery percentage display (tmux-battery)
- System info formatters

**Why own it**:
- Simple enough to understand completely
- OS-specific anyway (your scripts only need to work on macOS)
- Easy to modify for your exact needs
- No external dependencies beyond standard tools

**How to vendor**:
- Copy the script
- Strip out OS detection and other-OS code paths
- Simplify to your exact needs
- Add comments explaining what it does
- Put in `~/.tmux/scripts/`

**Example**: Battery script
```bash
# Instead of: tmux-plugins/tmux-battery (12 files, 500+ lines)
# Just do:
#!/bin/bash
pmset -g batt | grep -Eo "\d+%" | cut -d% -f1
```

## Category 3: Complex Libraries (Vendor Carefully)

**What it is**: Multi-file codebases with complex logic, but pure-shell/script

**Examples**:
- tmux-fzf (interactive session/window picker)
- Complex status bar formatters

**Why vendor it**:
- More than you want to write from scratch
- But still understandable with AI help
- No runtime dependencies (just shell)
- Stable (not updating frequently)

**How to vendor**:
- Copy the whole thing into your repo
- Add clear attribution
- Document what it does and why you use it
- Simplify over time as you understand it

**When NOT to vendor**:
- If you don't actually use it
- If you can write equivalent functionality simply
- If it has frequent security updates

## Category 4: Binary Tools (System Dependency)

**What it is**: Compiled tools, system utilities

**Examples from your setup**:
- tmux itself
- fzf
- zoxide
- gum

**Why NOT vendor**:
- Binary compilation complexity
- Security updates matter
- System package managers handle them well
- Truly complex codebases

**How to handle**:
- List in Brewfile
- Document minimum version if needed
- Don't try to include in dotfiles

## Category 5: Runtime/Platform (System Dependency)

**What it is**: The OS, shell, core utilities

**Examples**:
- macOS
- zsh/bash
- coreutils
- system commands (pmset, top, etc.)

**Why NOT vendor**:
- Can't vendor an OS
- System-level security updates
- Foundation of everything else

**How to handle**:
- Document requirements ("macOS 13+", "zsh 5.8+")
- Use standard tools when possible
- Add compatibility checks if needed

## The Plugin Classification

Let's classify your current tmux plugins:

### tmux-cpu, tmux-battery
- **Category**: Simple Scripts
- **Verdict**: Own it - rewrite or vendor and simplify
- **Reason**: <50 lines of actual logic, macOS-specific anyway

### catppuccin
- **Category**: Pure Config
- **Verdict**: Own it - copy the colors you want
- **Reason**: Just color definitions and formatting

### tmux-fzf
- **Category**: Complex Library
- **Verdict**: Keep as dependency OR find simpler alternative
- **Reason**: Interactive picker with real logic
- **Alternative**: Use native tmux choose-tree or write simple picker

### tmux-sensible
- **Category**: Pure Config
- **Verdict**: Own it - it's just default settings
- **Reason**: Literally just sets sane defaults (escape-time, etc.)
- **Action**: Copy the settings you agree with to your tmux.conf

### tmux-yank
- **Category**: Simple Scripts
- **Verdict**: Check if you need it
- **Reason**: Clipboard integration - tmux has built-in clipboard support now
- **Action**: Test if `set -g set-clipboard on` is sufficient

### tmux-sidebar
- **Category**: Simple Scripts
- **Verdict**: Do you use this?
- **If yes**: Vendor and simplify
- **If no**: Remove

### tmux-harpoon
- **Category**: Simple Scripts
- **Verdict**: Check usage
- **Reason**: Quick window navigation - but you have keybindings for this

### tmux-prefix-highlight
- **Category**: Simple Scripts
- **Verdict**: Own it or remove
- **Reason**: Shows when prefix is active - nice to have but trivial to implement

### tmux-pop
- **Category**: Simple Scripts
- **Verdict**: Check if needed
- **Reason**: Popup windows - tmux has native popups now

### TPM (plugin manager)
- **Category**: Infrastructure
- **Verdict**: Remove entirely
- **Reason**: No plugins = no plugin manager needed

## The Ownership Decision Tree

```
Do you actually use this feature?
├─ No → Remove it
└─ Yes
   └─ Is it just config/values?
      ├─ Yes → Copy values to your config
      └─ No
         └─ Is it <100 lines of script?
            ├─ Yes → Vendor or rewrite
            └─ No
               └─ Does it have real complexity/value?
                  ├─ Yes → Vendor with attribution, plan to simplify
                  └─ No → Rewrite simpler version
```

## The "Just Write It" Test

Before vendoring any code, ask: **Could I write this in an afternoon with Claude's help?**

If yes: Write it.
If no: Are you sure you need it?

Most tmux plugins fail this test - they're general-purpose solutions to simple problems.

### Example: tmux-sensible

The plugin:
- Dozens of files
- Cross-platform compatibility
- Update mechanism
- Documentation

Your version:
```tmux
# Sane defaults
set -sg escape-time 0
set -g history-limit 50000
set -g display-time 4000
set -g status-interval 5
set -g focus-events on
setw -g aggressive-resize on
```

6 lines. That's the "plugin".

### Example: Status bar with CPU and battery

The plugin approach:
- tmux-cpu plugin (complex script with GPU support, multiple OS, customization)
- tmux-battery plugin (similar complexity)
- Catppuccin wrapper (theming layer)
- TPM (installation and management)

Your version:
```bash
#!/bin/bash
cpu=$(top -l 2 -n 0 -F -s 0 | grep "CPU usage" | tail -1 | awk '{print $3}')
bat=$(pmset -g batt | grep -Eo "\d+%")
echo " $cpu  $bat  $(date +'%H:%M')"
```

8 lines. That's the "plugins".

## When External Makes Sense

There ARE cases where external dependencies make sense for tmux:

### 1. Binary Tools You Integrate With
- fzf (for session picking)
- ripgrep (for searching scrollback)
- gum (for interactive prompts)

These are real tools with complex implementations. Use them via Homebrew.

### 2. Your Own Separate Projects
If you write a tool that's used by multiple projects (not just tmux), make it separate:
- A sophisticated session manager
- A custom launcher system
- Shared utilities

But note: Even these can be in your dotfiles repo if they're personal tools.

### 3. Actual Libraries (If You're Using a Real Language)

If you were writing tmux plugins in Python/Ruby/whatever with actual dependencies:
- Use that language's package manager
- Pin versions
- Document in requirements.txt or equivalent

But tmux plugins are shell scripts. Shell doesn't have dependency management. Don't pretend it does with git submodules.

## The Submodule Trap

Git submodules feel like "proper dependency management" but they're actually:

**Worst of both worlds**:
- ❌ Not automatic (must init/update manually)
- ❌ Not versioned (points to commit, but that commit can disappear)
- ❌ Not isolated (pollutes your repo structure)
- ❌ Not simple (adds cognitive overhead)

**And you get none of the benefits**:
- ❌ Not actually getting updates (you're pinned to a commit)
- ❌ Not getting security patches (unless you manually update)
- ❌ Not simplified (still have to understand the code when it breaks)

**You're paying all the cost of a dependency with none of the benefits.**

## The New Philosophy

For tmux config in 2025:

### Tier 1: Your Code (90% of functionality)
- Config settings
- Simple scripts
- Keybindings
- Colors/theme

**How**: Directly in your dotfiles, written by you (with AI help)

### Tier 2: System Tools (9% of functionality)
- fzf
- tmux itself
- Shell utilities

**How**: Homebrew/system packages, listed in Brewfile

### Tier 3: Complex Vendored Code (1% of functionality)
- Only if genuinely complex AND genuinely needed
- Fully copied into your repo with attribution
- Plan to simplify or replace over time

**How**: Copied to vendor/ with README explaining why

### Tier 4: Removed (Former "plugins")
- Everything else
- All the plugin managers
- All the submodules

**How**: Delete it

## Conclusion

Dependencies should be:
1. **Necessary** (can't easily write yourself)
2. **Maintained** (active security updates if needed)
3. **Isolated** (clear boundaries)
4. **Understood** (you know what they do)

Tmux plugins are:
1. ❌ Unnecessary (trivial to reimplement)
2. ❓ Maintained (maybe, but do you need updates?)
3. ❌ Not isolated (submodules in your repo)
4. ❌ Not understood (you're debugging plugin infrastructure)

The solution: Stop depending. Start owning.

With AI assistance, the cost of ownership approaches zero.
The cost of dependencies remains constant.

Own your stack.
