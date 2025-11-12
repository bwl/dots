# Plugin Architecture Philosophy: Pre-AI vs AI-Assisted Development

## The Historical Context

### Why Plugin Managers Existed (2010-2024)

The tmux plugin ecosystem (TPM, plugins as git repos, etc.) emerged to solve specific problems:

1. **Code Sharing at Scale**: One developer writes code, thousands benefit
2. **Update Distribution**: Bug fixes propagate to all users automatically
3. **Maintenance Burden**: No single user needs to understand all the code
4. **Discovery**: Centralized lists of "best" solutions
5. **Standardization**: Common patterns across different users' setups

This model made sense when:
- Individual developers couldn't easily understand or modify complex code
- Writing new features was time-consuming
- Bugs were hard to fix without deep expertise
- Each person maintaining their own fork was impractical

### The AI Paradigm Shift (2025+)

With AI coding agents, the economics fundamentally change:

**What's Now Trivial:**
- Understanding any codebase ("explain this plugin to me")
- Forking and customizing ("change this to use different colors")
- Fixing bugs ("this breaks on macOS, fix it")
- Implementing features from scratch ("write a status bar that shows CPU")
- Adapting code to your exact needs ("make this work with my theme")

**What's Still Valuable:**
- Good design patterns (AI can implement, but you still need vision)
- Tested, working implementations (as reference or starting point)
- Creative ideas (AI executes, you ideate)

## The Core Question: Do You Need Upstream?

For a tmux plugin, ask:
1. **Do you modify it?** If yes, upstream updates are a liability (merge conflicts)
2. **Does it need security updates?** Tmux plugins are display-only shell scripts - low risk
3. **Do you want new features from upstream?** Or do you want stability?
4. **Is the code complex enough to maintain separately?** Most plugins are <500 lines

### The Reality Check

Look at tmux-cpu:
- 2,401 bytes (cpu.tmux)
- Calls simple shell scripts
- Last meaningful update: probably years ago
- Your needs: show CPU percentage in your status bar

**Do you need git submodule tracking of upstream tmux-plugins/tmux-cpu?** No.
**Do you need a plugin manager to install it?** No.
**Do you need the code?** Yes - but as YOUR code, adapted to YOUR needs.

## The 2025+ Model: Code as Raw Material

### Principle 1: Internalize What You Use

If you use code, bring it into your repo as first-class code:
- No submodules pointing to external repos
- No plugin managers
- No "vendor" directories
- Just regular code in your config structure

### Principle 2: Attribution Over Dependency

Replace runtime dependencies with attribution:
```bash
# From: https://github.com/tmux-plugins/tmux-cpu
# Original license: MIT
# Modified for personal use
```

You're not hiding the origin, you're acknowledging it while taking ownership.

### Principle 3: Simplicity Over Generality

External plugins are general-purpose (support many users, many OS, many configs).
Your code should be specific-purpose (support YOU, YOUR OS, YOUR config).

tmux-cpu supports:
- Multiple OS (Linux, macOS, BSD, Windows)
- Multiple percentage display formats
- Multiple icon sets
- Customizable colors
- GPU monitoring
- RAM monitoring
- Temperature monitoring

You need:
- macOS
- CPU percentage
- One specific format that matches your status bar

**The general version might be 500 lines. Your version might be 20.**

### Principle 4: AI as Maintenance Partner

Instead of:
1. Run `git submodule update`
2. Hope nothing breaks
3. Debug compatibility issues

You do:
1. "Claude, update my CPU status to use less CPU when calculating"
2. Done

The AI understands your specific code, your specific setup, your specific needs.

## The Philosophical Shift

### Old Model: Consumer of Code
- You consume plugins
- Authors maintain plugins
- You adapt your config to fit plugins
- Updates happen TO you

### New Model: Owner of Code
- You own all code in your dotfiles
- AI helps you maintain it
- Plugins shape themselves to fit your config
- Updates happen BY you (via AI)

### Open Source Compatibility

This doesn't violate open source principles:
- MIT/BSD licenses explicitly allow vendoring and modification
- You attribute the source
- You could even publish your fork (though why would you? it's personal config)
- The original authors still have their repo for others to use

You're not rejecting the community - you're graduating from consumer to owner.

## Practical Implications for Tmux

### What Should Be External?
- **TPM**: Remove entirely. You don't need a plugin manager.
- **Plugins as submodules**: Remove. Bring code in or rewrite.

### What Should Be Internal?
All functionality you actually use:
- Status bar components (CPU, battery, date/time, etc.)
- Key bindings and navigation
- Theme/colors
- Window/pane behavior

### What Should Be Simple Scripts?
Most "plugins" are just:
1. A tmux config snippet
2. A few shell scripts
3. Some color definitions

These can be:
- `~/.tmux/scripts/cpu.sh` (the actual script)
- `~/.tmux.conf` (calls the script in status-right)

No .tmux file, no plugin registration, no installation step.

## The Test: Can You Understand It?

If you can understand the code well enough to maintain it, you should own it.
If you can't understand it... should you be running it?

With an AI partner, "can you understand it" becomes:
- "Claude, explain what this code does"
- "Claude, should I be worried about this?"
- "Claude, how would I modify this?"

Understanding is now a conversation, not a prerequisite.

## Conclusion

The question isn't "should I vendor this plugin?"
The question is "why wouldn't I?"

In 2025, with AI assistance:
- You can maintain any code you can describe
- You can modify any code you can read
- You can own any dependency you can understand

The plugin ecosystem was a solution to human limitations.
Those limitations are rapidly disappearing.

Your dotfiles should reflect YOUR choices, maintained by YOUR AI partner, for YOUR use case.

Not a collection of submodules to someone else's choices.
