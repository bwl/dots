# Zellij vs Tmux: Deep Evaluation for Your Use Case

## Context: What You Need

**Primary use case**: Laptop + one terminal, project-based AI agent workspace

**Requirements**:
1. Quick session creation per project
2. Multiple agents/tools in tabs/windows
3. Task list visibility (on-demand)
4. Simple, maintainable config
5. Fast keyboard navigation
6. Ownership of your setup

## The Plugin Problem (Why We're Here)

Your tmux status bar is broken because of plugin dependency hell:
- TPM (plugin manager)
- 10+ git submodules
- Complex load ordering
- Literal `#{cpu_percentage}` instead of actual value

**Core question**: Is this a tmux problem or a "how we used tmux" problem?

## Zellij Pros

### 1. Modern Design Philosophy
**Batteries included, not assembly required**

- Status bar works out of the box (no plugins needed)
- Floating panes built-in (no tmux-popup plugin)
- UI elements are first-class (no hacky shell scripts)
- Auto-updates status bar (CPU, memory, etc. via plugins, but official ones)

**What this means for you**: Less configuration to own, more features that just work.

### 2. Better Configuration Language
**KDL vs tmux.conf**

Tmux:
```tmux
set -g status-right "#(~/.tmux/scripts/status.sh)"
bind -n M-Left select-pane -L
%hidden thm_bg="#1e1e2e"
```

Zellij:
```kdl
keybinds {
    normal {
        bind "Alt Left" { MoveFocus "Left"; }
    }
}
```

**What this means**: More readable, structured, less arcane syntax. But also less mature (fewer examples online).

### 3. Floating Panes Are Native
**Not a plugin, not a hack**

Tmux floating panes require:
- tmux 3.2+
- Manual popup management
- Shell script wrapper
- No persistence across sessions

Zellij floating panes:
- Built-in, well-designed
- Configurable size/position in layout
- Toggle visibility with keybind
- Part of the layout system

**For your taskbook use case**: Perfect. Ctrl-T to show/hide tasks is exactly what floating panes are for.

### 4. Layouts Are Data
**Not shell scripts, actual declarative layouts**

Tmux layout approach:
```bash
# Shell script that creates windows and sends keys
tmux new-window -n "claude"
tmux send-keys "claude" C-m
```

Zellij layout approach:
```kdl
tab name="claude" {
    pane command="claude"
}
```

**What this means**:
- Layouts are reproducible
- Less brittle (no "send keys and hope")
- Easier to understand and modify
- Can be version controlled clearly

### 5. Active Development
**Modern codebase, active community**

- Written in Rust (memory-safe, fast)
- Released in 2021, actively maintained
- Modern terminal features (true color, etc.)
- Plugin system via WASM (proper isolation)

**What this means**: Future-oriented, won't become abandonware.

### 6. No Plugin Manager Needed
**Plugins are WASM, not git submodules**

Official plugins ship with Zellij. Custom plugins are:
- Compiled to WASM
- Sandboxed
- Versioned properly
- Don't pollute your dotfiles

**What this means**: You won't have the TPM problem. If you need a plugin, it's a proper dependency, not a submodule.

## Zellij Cons

### 1. Young Ecosystem
**Less battle-tested, fewer resources**

Tmux (2007, 18 years old):
- Thousands of blog posts
- Every edge case documented
- Mature plugin ecosystem
- Works everywhere

Zellij (2021, 4 years old):
- Growing documentation
- Smaller community
- Fewer examples for complex setups
- Might hit undocumented bugs

**Risk for you**: When something breaks, less StackOverflow help.

### 2. Different Mental Model
**Tabs, panes, floating panes - not windows**

Tmux: Sessions → Windows → Panes
Zellij: Sessions → Tabs → Panes + Floating Panes

It's similar but not identical. Muscle memory doesn't transfer perfectly.

**Impact for you**: Learning curve. Your fresh script logic needs rethinking.

### 3. Command Execution in Layouts
**How well does `pane command="claude"` work?**

Tmux approach:
```bash
# Source .zshrc, run command
tmux send-keys "source ~/.zshrc && claude" C-m
```

Zellij approach:
```kdl
pane command="claude"
```

**Unknown**: Does Zellij properly inherit your shell environment? Will `claude` (an alias) work?

Possible issues:
- Aliases might not be available
- PATH might not include custom bins
- Might need wrapper script: `pane command="zsh" args "-ic" "claude"`

**This needs testing.**

### 4. Floating Pane Limitations
**Can you run auto-refreshing commands?**

Your taskbook sidebar in tmux:
```bash
while true; do tb; clear; sleep 2; done
```

Can this work in a Zellij floating pane? Unknowns:
- Does the pane stay open with a loop?
- Can you background it and toggle visibility?
- What happens when you switch tabs?

**This is critical for your use case and needs testing.**

### 5. Session Management Maturity
**Does it handle your workflow?**

Your tmux workflow:
1. `cd project`
2. `fresh` → creates session named after project
3. Work in session
4. `fresh kill` → kills session
5. Later: `cd project && fresh` → reattaches

Zellij equivalent needs:
- Session names based on current directory
- Ability to list sessions
- Ability to attach by name
- Ability to kill from within

**Probably works, but needs testing.**

### 6. Less Control Over Status Bar
**More opinionated, less customizable**

Tmux status bar:
- Total control via format strings and scripts
- You own every character
- Broken plugin ecosystem, but you can fix it

Zellij status bar:
- Built-in, looks good
- Customization via plugins (WASM)
- Less direct control

**For you**: Probably fine (you wanted less config anyway), but if you need custom status items, it's harder.

## The Ownership Question

**Tmux**: You can own everything (shell scripts, simple config), but the ecosystem pushes you toward plugins.

**Zellij**: Less to own (features built-in), but what you do own is cleaner (KDL, layouts).

### What Can You Own in Zellij?

**Fully owned**:
- Layout files (KDL, readable)
- Keybindings (KDL config)
- Session launcher script (bash, same as tmux)

**Partially owned**:
- Status bar (can customize via themes, not scripts)
- Plugin configuration (if you need custom plugins)

**Not owned** (but don't need to be):
- Core UI (tabs, panes, floating panes)
- Status bar plugins (official, maintained)

### What Can You Own in Tmux (Post-Plugin-Removal)?

**Fully owned**:
- Config file (tmux.conf, arcane but learnable)
- Status bar scripts (shell scripts)
- Layout scripts (bash)

**Not owned**:
- Core UI (but it's simple and stable)

## The Real Question: Which Philosophy Fits?

### Tmux Philosophy
**"Simple, stable, unopinionated multiplexer"**

You build your UI with shell scripts and config. Full control, full responsibility.

**Good for**: Power users who want total ownership, minimal dependencies.

**Bad for**: People who want modern features without building them.

### Zellij Philosophy
**"Modern terminal workspace with batteries included"**

You get a designed experience. Less to configure, more that just works.

**Good for**: Users who want a polished tool, don't mind some opinions.

**Bad for**: Control freaks who need to own every pixel.

## Your Specific Use Case Analysis

### What You Actually Do
- Launch fresh in a project directory
- Switch between 4 agent windows
- Glance at taskbook occasionally
- Don't customize much (you want it to just work)

### Tmux Fit: 7/10
**Pros**:
- You know it
- Fresh script works
- Total control if needed

**Cons**:
- Plugin mess (current problem)
- Arcane config syntax
- Have to build floating panes yourself

### Zellij Fit: 8.5/10
**Pros**:
- Floating panes perfect for taskbook
- Cleaner config
- Modern, active development
- Less to maintain

**Cons**:
- Need to test command execution
- Smaller community
- Slight learning curve

## Decision Framework

### Test First
**Don't commit yet. Build the Zellij version and test:**

1. Create fresh.kdl layout
2. Test if `pane command="claude"` works (or needs wrapper)
3. Test floating pane with auto-refresh taskbook
4. Test session management script
5. Use it for a real project for 2-3 days

**If all tests pass**: Zellij is probably better for you.
**If floating pane auto-refresh doesn't work**: Dealbreaker, stick with tmux.
**If command execution is clunky**: Might be fixable, but adds complexity.

### The Hybrid Option
**Keep both, choose per-context**

- **Zellij**: Primary for local laptop work (your main use case)
- **Tmux**: Backup for servers, ssh sessions, edge cases

This is actually smart:
- Zellij is newer, better for local interactive work
- Tmux is ubiquitous, better for remote sessions
- You learn both, get benefits of both

## Recommendation

**Try Zellij. Here's why:**

1. **Your main problem is plugin complexity** - Zellij doesn't have that
2. **Floating panes are perfect for your taskbook UX** - Native support
3. **You want less config to maintain** - Zellij gives you more for free
4. **You're already exploring alternatives** - The itch to switch is real
5. **Worst case, you have tmux as fallback** - Low risk

**But test these dealbreakers first:**
1. Can floating panes run auto-refresh loops?
2. Can panes execute shell aliases/functions?
3. Is session management smooth?

If any of those fail, tmux with simplified config is still the right answer.

## The "Own Your Stack" Perspective

**Tmux path**: Remove all plugins, write simple scripts, own everything
- Aligns with ownership philosophy
- More work upfront
- Total control

**Zellij path**: Use built-in features, own layouts and launcher
- Less to own, but what you own is cleaner
- Less work upfront
- Some abstraction (but well-designed)

**Both are valid.** Zellij isn't "giving up ownership" - it's "choosing better abstractions to own."

Owning shell scripts that manage tmux plugins is not inherently better than owning KDL layouts that configure Zellij features.

The question is: which abstractions are better?

For your use case (AI agent workspace), Zellij's abstractions (tabs, floating panes, layouts) seem better than tmux's (windows, manual popups, shell scripts).

## Next Steps

1. **Write the Zellij implementation** (layout, config, script)
2. **Test the dealbreakers** (floating pane refresh, command execution)
3. **Use it for real work** (3-5 days)
4. **Decide**:
   - If it works well: Adopt Zellij as primary
   - If it has issues: Fix tmux properly (remove plugins, simple scripts)
   - If unsure: Keep both

Ready to build the Zellij version?
