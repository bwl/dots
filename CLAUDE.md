# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

Personal macOS dotfiles repository with automated setup, Bitwarden CLI secret management, and tmux-based AI agent workspace. Manages 55 Homebrew packages, shell configurations (zsh + Powerlevel10k/Zim), and application configs for development tools. Includes RETRO-OS Phase 0 tools for enhanced tmux session management.

## Common Commands

### Setup & Installation

```bash
# Fresh system bootstrap (installs everything)
./install.sh

# Create/update symlinks only
./scripts/setup_symlinks.sh --force

# Apply macOS system preferences
./macos-defaults.sh
```

### Package Management

```bash
# Update package lists from current system
brew bundle dump --force --file=Brewfile
mas list > Masfile
npm list -g --depth=0 | tail -n +2 | sed 's/^[├└]─* //' > npm-globals.txt
ls ~/.cargo/bin/ | grep -v "^cargo\|^rust" > cargo-installs.txt
uv tool list | grep -E "^\S" | awk '{print $1}' > uv-tools.txt

# Install packages from lists
brew bundle install --file=Brewfile
cat npm-globals.txt | grep -v "^#" | xargs npm install -g
cat cargo-installs.txt | grep -v "^#" | xargs cargo install
cat uv-tools.txt | grep -v "^#" | xargs -I {} uv tool install {}

# Install Go tools (RETRO-OS Phase 0)
go install github.com/joshmedeski/sesh@latest

# Restore VS Code extensions
cat vscode-extensions.txt | xargs -L 1 code --install-extension
```

### Hearth (tmux + Claude)

```bash
cd ~/Developer/my-project
hearth              # Start tmux session with claude (enables canvas)
hearth --help       # Show help
```

Creates tmux session named after current directory, runs Claude Code inside.
Enables claude-canvas skill (requires tmux for split panes, Bun for canvas CLI).
If already in tmux, just runs claude directly.

### Experiment Directory Management (Try)

```bash
# Browse all experiments
try

# Search for or create experiment directory (date-prefixed)
try redis

# Create new experiment with today's date
try new api

# Clone repository into dated directory
try clone https://github.com/user/repo

# Create git worktree for parallel development
try worktree /path/to/main-repo experiment-name
```

Try creates date-prefixed directories (e.g., `2025-11-12-redis-cache`) in `~/src/tries` for organizing experiments. Shell integration is in `.zshrc.d/integrations.zsh`. Manual installation:

```bash
curl -sL https://raw.githubusercontent.com/tobi/try/refs/heads/main/try.rb > ~/.local/try.rb
chmod +x ~/.local/try.rb
```

## Architecture

### Directory Structure

```
dotfiles/
├── .claude/            # Claude Code configuration (symlinked to ~/.claude/)
│   ├── skills/        # Custom Claude Code skills
│   │   └── asker/     # AskUserQuestion-based requirement gathering
│   ├── CLAUDE.md      # Global Claude instructions (ast-grep preference)
│   └── settings.local.json  # Local Claude settings
├── shell/              # Zsh configs (modular .zshrc.d pattern)
│   ├── .zshrc         # Modular loader (sources .zshrc.d/*.zsh)
│   ├── .zshrc.d/      # Modular config files
│   ├── .zshenv        # Minimal env setup (PATH, secrets)
│   └── .p10k.zsh      # Powerlevel10k theme (auto-generated)
├── config/             # App configs (symlinked to ~/.config/)
│   ├── nvim/          # LazyVim setup
│   ├── ghostty/       # Terminal emulator config + shaders
│   ├── sketchybar/    # Custom macOS status bar
│   ├── tmux/          # tmux.conf + custom menu
│   ├── karabiner/     # Keyboard customization
│   ├── gh/            # GitHub CLI config
│   ├── fd/            # fd config
│   └── yazi/          # File manager config
├── vendor/             # Cloned external tools (gitignored)
│   ├── tmuxifier/     # Tmux layout manager (symlinked to ~/.tmuxifier)
│   └── cleanup-cache/ # Cache cleanup tool (binary: tidyup)
├── git/                # .gitconfig + .gitignore_global
├── ssh/                # SSH config (NOT keys!)
├── vscode/             # VS Code settings.json
├── pake/               # Pake desktop app wrappers (X, GitHub)
├── bin/                # User scripts (hearth, automux)
├── scripts/
│   └── setup_symlinks.sh         # Symlink management
├── app-configs/        # Non-XDG app configs (Syncthing, Obsidian notes)
├── docs/               # Documentation (tmux guides, enhancements)
├── Brewfile            # Homebrew packages (55 formulae)
├── Masfile             # Mac App Store apps (10 apps)
├── npm-globals.txt     # Global npm packages (8 packages)
├── cargo-installs.txt  # Cargo-installed binaries (5 packages)
├── uv-tools.txt        # uv tool installs (3 packages)
├── vscode-extensions.txt  # VS Code extensions (108 extensions)
├── install.sh          # Bootstrap script for fresh macOS
└── macos-defaults.sh   # System preferences automation
```

### Historical/Archived Directories

These directories exist in the repo but are not actively used:

- **`claude-thoughts/`** - Archive of tmux multiplexer analysis (12 markdown files, ~176KB). Historical exploration from hearth development. Read-only reference.

- **`.taskbook/`** - Legacy taskbook data. Can be safely ignored.

- **`wiki/`** - GitHub wiki content (gitignored). Maintained separately in GitHub wiki. Local copy for offline reference only.

### Shell Configuration (Modular)

Uses **Zim** framework with Powerlevel10k theme. Configuration split into modular files:

**File structure**:
- `shell/.zshrc` - Loader that sources `.zshrc.d/*.zsh` (Claude Code sessions get minimal inline config)
- `shell/.zshrc.d/` - Modular configs:
  - `core.zsh` - History, keybindings, Zim framework init
  - `path.zsh` - PATH management
  - `env.zsh` - Environment variables (HOMEBREW, FZF, GOPATH)
  - `aliases.zsh` - Aliases (claude, tarot)
  - `functions.zsh` - Shell functions
  - `completions.zsh` - Command completions (crush, cliffy, docker)
  - `integrations.zsh` - Tool integrations (pyenv, zoxide, broot, Try)
- `shell/.zshenv` - Minimal bootstrap (basic PATH, secrets loading)
- `shell/.p10k.zsh` - Powerlevel10k theme (auto-generated by `p10k configure`)

**Key settings**:
- **History**: 10M entries, shared across sessions, deduplication enabled
- **Keybindings**: Emacs mode (`bindkey -e`)
- **Modules**: zsh-autosuggestions, zsh-syntax-highlighting, history-substring-search
- **Pyenv**: Uses `--no-rehash` for faster startup

**Custom functions** (in `functions.zsh`):
- `tb()` - Smart taskbook wrapper (uses local `.taskbook` if present)
- `git()` - Wrapper with retry logic for `index.lock` issues using `attempt` CLI
- `cdx()` - Codex wrapper with update capability
- `glowf()` - Markdown file finder/viewer with fzf + glow
- `codex_safe()` - Run codex with confirmation before executing

### Symlink Strategy

The `setup_symlinks.sh` script creates symlinks from `~/dotfiles/` to standard locations:

1. **Shell files**: `shell/.zshrc` → `~/.zshrc` (etc.)
2. **Git configs**: `git/.gitconfig` → `~/.gitconfig`
3. **SSH config**: `ssh/config` → `~/.ssh/config`
4. **Config dirs**: Each `config/*` → `~/.config/*` (individual symlinks per directory)
5. **Tmuxifier**: `vendor/tmuxifier` → `~/.tmuxifier` (from vendor/, not ~/.config/)
6. **Claude Code**: `.claude/skills/` → `~/.claude/skills/`, `.claude/CLAUDE.md` → `~/.claude/CLAUDE.md`, `.claude/settings.local.json` → `~/.claude/settings.local.json`
7. **User bin**: `bin/` → `~/bin/`
8. **VS Code**: `vscode/settings.json` → `~/Library/Application Support/Code/User/settings.json`

Use `--force` flag to backup existing files before replacing. Backups stored in `~/dotfiles_backup_YYYYMMDD_HHMMSS/`.

**Note**: `~/.claude/` contains runtime data (history, sessions, debug logs) that should NOT be symlinked. Only skills, settings, and global instructions are managed in dotfiles.

### Secret Management

**Critical**: This repo uses Bitwarden CLI for secrets. NEVER commit:
- API keys, tokens, passwords
- SSH private keys (`id_*`, `*.pem`, `*.key`)
- `.env` files
- Bitwarden session keys

Secrets are referenced in shell configs via:
```bash
export MY_API_KEY="$(bw get notes 'Secret Name')"
```

Current secrets to migrate (see `shell/.zshenv`):
- `CLIFFY_OPENROUTER_API_KEY` - should be moved to Bitwarden

### Hearth

Simple tmux wrapper (`~/bin/hearth`) for running Claude Code with canvas support:

1. **Session naming**: Derives from current directory (sanitized: `.` and ` ` → `_`)
2. **Attach or create**: Attaches to existing session, or creates new one running claude
3. **Canvas enablement**: tmux provides split-pane capability for claude-canvas skill
4. **Passthrough**: If already in tmux, just runs claude directly

### SketchyBar Configuration

SketchyBar is a highly customizable macOS status bar that replaces the native menu bar.

**Location**: `config/sketchybar/` (symlinked to `~/.config/sketchybar/`)

**Structure**:
- `sketchybarrc` - Main configuration file (bash script)
- `colors.sh` - Color scheme definitions
- `icons.sh` - Nerd Font icon definitions
- `plugins/` - Custom plugin scripts for status bar items

**Common commands**:
```bash
# Reload configuration
sketchybar --reload

# Start/stop service
brew services start sketchybar
brew services stop sketchybar
brew services restart sketchybar

# Debug mode (run in foreground)
sketchybar --config ~/.config/sketchybar/sketchybarrc
```

**Included plugins** (minimal starter config):
- `space.sh` - Workspace/desktop indicators (10 spaces)
- `front_app.sh` - Currently focused application
- `clock.sh` - Time and date display
- `battery.sh` - Battery status with charging indicator
- `network.sh` - WiFi connection and SSID
- `cpu.sh` - CPU utilization percentage
- `memory.sh` - RAM usage percentage

**Creating custom plugins**:
1. Create executable script in `~/.config/sketchybar/plugins/`
2. Use environment variables: `$NAME`, `$INFO`, `$SENDER`, `$SELECTED`
3. Update items via `sketchybar --set $NAME label="value"`
4. Reference in `sketchybarrc` with `script="$PLUGIN_DIR/your_plugin.sh"`

**Color customization**: Edit `colors.sh` to match your theme preference (currently using dark Catppuccin-inspired palette)

**Auto-hide functionality**:
- SketchyBar automatically hides when cursor moves to top screen edge (to reveal macOS menu bar)
- Uses dual-threshold hysteresis to prevent flickering (matches macOS menu bar behavior)
- **Cursor bouncer** prevents accidental menu bar access by "bouncing" cursor at 5px from top
- Hold Command key to bypass bouncer and access menu bar
- Menu detection keeps bar hidden while macOS menus are open
- Implemented via lightweight Swift cursor monitor (`helpers/cursor_monitor.swift`)
- Monitor sends custom SketchyBar events: `cursor_at_top` / `cursor_away_from_top`
- Plugin (`plugins/auto_hide.sh`) toggles bar visibility in response
- Monitor starts automatically with SketchyBar, no additional setup needed
- Disable: Comment out "Auto-Hide Functionality" section in `sketchybarrc`

**Architecture**:
- `helpers/cursor_monitor.swift` - Native Swift script, polls cursor position using AppKit/CoreGraphics (no dependencies)
- **Cursor bouncer**: Uses `CGWarpMouseCursorPosition()` to prevent cursor from entering top 5px zone
- Bouncer disabled when Command key held (`NSEvent.modifierFlags.contains(.command)`)
- Dual thresholds: `topEdgeThreshold = 3.0` (hide), `bottomEdgeActiveThreshold = 44.0` (show)
- Hysteresis prevents flickering: bar hides at top 3px, only reappears when cursor moves below 44px
- Creates 3-44px "dead zone" for smooth macOS menu bar interaction
- Menu detection via `CGWindowListCopyWindowInfo` (window level ≥ 101 = menu open)
- Sends SketchyBar events only on state change (no redundant triggers)
- `plugins/auto_hide.sh` - Event handler that toggles `--bar hidden=on/off`
- Integrated in `sketchybarrc` with automatic process management

**Accessibility permissions**:
- Cursor bouncer requires macOS Accessibility permissions to move cursor
- Grant in: System Settings > Privacy & Security > Accessibility > Enable for Terminal/Swift
- If denied: Bouncer disabled, auto-hide still works (read-only cursor tracking)
- Check permissions: `sketchybar --reload` shows warning if not granted

**Customization**:
- Adjust thresholds in `cursor_monitor.swift` lines 11-14:
  - `topEdgeThreshold`: Auto-hide trigger distance (3px default)
  - `bottomEdgeActiveThreshold`: Auto-show trigger distance (44px default)
  - `bouncerThreshold`: Cursor bounce distance (5px default)
  - `bounceTargetOffset`: Where to bounce cursor back to (6px default)
- Values tuned for MacBook Air at 1710x1112 resolution
- May need adjustment for different displays/DPI

### Claude Code Skills

Custom skills extend Claude Code's capabilities. Managed in `.claude/skills/` (symlinked to `~/.claude/skills/`).

**Available skills**:

- **asker** (`~/.claude/skills/asker/`) - Interactive requirement gathering using AskUserQuestion tool
  - Resolves ambiguities through structured multi-choice questions (2-4 options)
  - Auto-invoked when user requests are unclear or multiple approaches exist
  - Covers: technology selection, architecture decisions, feature prioritization, implementation trade-offs
  - Files: `SKILL.md` (main definition), `examples.md` (6 scenarios), `templates.md` (10 patterns), `README.md`
  - Tool restriction: `allowed-tools: AskUserQuestion` (read-only interaction)

**Creating new skills**:

1. Create directory: `mkdir -p ~/dotfiles/.claude/skills/my-skill`
2. Add `SKILL.md` with YAML frontmatter:
   ```yaml
   ---
   name: my-skill
   description: When and why to use this skill (max 1024 chars)
   allowed-tools: Read, Grep, Glob  # Optional: restrict tools
   ---
   ```
3. Add markdown instructions below frontmatter
4. Run `./scripts/setup_symlinks.sh --force` to symlink to `~/.claude/skills/`
5. Restart Claude Code session to register skill

**Skill guidelines**:
- Keep focused (single responsibility)
- Clear description for model invocation
- Use `allowed-tools` to restrict capabilities when appropriate
- Add supporting files for examples, templates, reference docs
- Test by triggering relevant use cases in Claude Code

### Pake Desktop App Wrappers

[Pake](https://github.com/tw93/Pake) wraps websites into lightweight native desktop apps using Rust/Tauri.

**Location**: `pake/`

**Structure**:
- `Pakefile` - App manifest (Brewfile-style)
- `build.sh` - Build script
- `lib/harness.js` - Shared inject code
- `apps/<domain>/` - Per-site inject files (JS/CSS)

**Commands**:
```bash
# Build all apps
cd ~/dotfiles/pake && ./build.sh

# Build single app
./build.sh --app X
./build.sh --app GitHub
```

**Current apps**:
- **X** (`x.com`) - Twitter/X desktop wrapper
- **GitHub** (`github.com`) - GitHub desktop wrapper

**Adding apps**: Edit `Pakefile`, optionally create `apps/<domain>/inject.{js,css}`, run build.

**Output**: `~/Applications/Pake/`

**Requirements**: `npm install -g pake-cli`, Rust toolchain

## Development Notes

### Key Tools

- **Text editors**: neovim (LazyVim), msedit (retro terminal editor)
- **CLI search**: ripgrep (`rg`), fd, fzf, ast-grep (`sg`)
- **File managers**: yazi (primary)
- **Git UIs**: lazygit (TUI), tig, gh (CLI)
- **Monitoring**: htop, mactop, procs, ncdu
- **AI tools**: qwen-code, opencode, specify, crush
- **Task runners**: go-task, just
- **Language managers**: pyenv-virtualenv, uv (Python), bun (JavaScript), rust
- **Tmux enhancement (RETRO-OS Phase 0)**:
  - **zoxide**: Smart directory navigation (z command)
  - **gum**: Interactive shell script components
  - **sampler**: Terminal-based dashboard visualization
  - **sesh**: Tmux session manager (installed via `go install github.com/joshmedeski/sesh@latest`)
  - **tmuxifier**: Tmux layout/window manager (in vendor/, symlinked to ~/.tmuxifier)
  - **automux**: Automated tmux session launcher (cloned to ~/bin/automux)
  - **tmux-harpoon**: Quick tmux window navigation (TPM plugin)

### Custom Aliases

```bash
alias claude="$HOME/.claude/local/claude"
alias tarot="$HOME/Developer/tarot/tarot"
# Note: GNU grep available but commented out (use macOS grep by default)
```

### Completions

Dynamic completions loaded for: crush, cliffy, docker

External integrations: broot launcher, langflow environment

### Testing Changes

After modifying shell configs or scripts:

```bash
# Test in new shell without affecting current session
zsh -c 'source ~/.zshrc && your-test-command'

# Or reload current shell
exec zsh
```

After modifying symlinks:

```bash
# Verify symlink points to correct location
ls -la ~/.zshrc

# Check file resolves correctly
readlink ~/.zshrc
```

### Maintenance Tasks

**Update package lists** after installing new tools:
```bash
cd ~/dotfiles
brew bundle dump --force
# ... (other package list updates)
git add Brewfile Masfile npm-globals.txt cargo-installs.txt uv-tools.txt
git commit -m "Update package lists $(date +%Y-%m-%d)"
```

**Clean up Homebrew** periodically:
```bash
brew autoremove  # Remove unused dependencies
brew cleanup     # Remove old versions
brew doctor      # Check for issues
```

### Important Files Not to Modify

- `shell/.p10k.zsh` - Generated by Powerlevel10k configurator (`p10k configure`)
- `config/nvim/lazy-lock.json` - Managed by LazyVim package manager
- Backup files in `~/dotfiles_backup_*` - Created by setup_symlinks.sh

## Communication Guidelines

When writing documentation or communicating about this repository:

- **Sacrifice grammar for sake of concision** - prefer terse, direct language over full sentences
- **List unresolved questions at end, if any** - collect open questions/decisions in final section

## Additional Context

- **Total package count**: 55 Homebrew packages (81% reduction from original 296 packages)
- **Platform**: macOS (Darwin 25.0.0), tested on Apple Silicon and Intel
- **Git repo status**: Currently not a git repository (initialization instructions in README)
- **Documentation**: See `README.md` for comprehensive setup guide, `APPLICATIONS.md` for app-specific configs
- **RETRO-OS integration**: Phase 0 tools installed for enhanced tmux session management and navigation
