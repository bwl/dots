# Dotfiles Enhancement Ideas

Comprehensive roadmap for evolving this dotfiles repository into a world-class, self-contained development environment management system.

---

## 1. Global Theme Configuration System

**Concept**: Unified theming across all tools (terminal, shell, editors, CLIs) from a single source of truth.

**Problem**: Currently themes are scattered:
- Ghostty has its own theme/shaders
- Neovim has its own colorscheme
- Catppuccin used in tmux
- Powerlevel10k for shell prompt
- bat, fzf, eza each have their own color configs

**Solution**: Create `config/themes/` with theme definitions that cascade to all tools.

**Features**:
- Single theme file (YAML/TOML) defines colors once
- Generator scripts apply theme to all tools
- Support for multiple themes (dark/light variants)
- Per-machine theme overrides
- Theme preview before applying
- Automatically sync theme across Ghostty shaders, Neovim, tmux, bat, fzf, etc.

**Example Theme File** (`config/themes/catppuccin-mocha.yaml`):
```yaml
name: "Catppuccin Mocha"
variant: "dark"
colors:
  background: "#1e1e2e"
  foreground: "#cdd6f4"
  cursor: "#f5e0dc"
  selection: "#585b70"
  # ... all 26 ANSI colors
  accent: "#89b4fa"

tools:
  ghostty:
    shader: "catppuccin-mocha.glsl"
  nvim:
    colorscheme: "catppuccin-mocha"
  tmux:
    plugin_flavor: "mocha"
  bat:
    theme: "Catppuccin-mocha"
```

**Implementation**:
- Script: `scripts/apply-theme.sh <theme-name>`
- Generates tool-specific configs from theme YAML
- Updates `.zshrc`, `nvim/init.lua`, `ghostty/config`, etc.
- Validates theme compatibility before applying

**Priority**: Medium - Nice quality-of-life improvement

---

## 2. Self-Contained Zsh Dependencies

**Concept**: Internalize all external shell dependencies to make dotfiles truly portable and version-locked.

**Current External Dependencies**:
- Zim framework (downloaded from GitHub at runtime)
- Powerlevel10k theme (separate git clone)
- Completions: crush, cliffy, docker (dynamically sourced)
- Broot launcher (external script)
- Langflow environment (external source)

**Problems**:
- Dotfiles won't work offline on fresh install
- External repos can change/break
- No version locking
- Slower setup due to network downloads

**Solution**:

### Option A: Vendor Dependencies
```
~/dotfiles/
├── vendor/
│   ├── zim/              # Full Zim framework
│   ├── powerlevel10k/    # P10k theme
│   ├── completions/      # All completion scripts
│   │   ├── _crush
│   │   ├── _cliffy
│   │   ├── _docker
│   │   └── _gh
│   └── plugins/          # Other shell plugins
└── shell/
    └── .zshrc            # Sources from vendor/
```

**Update Process**:
```bash
# Update vendored dependencies
./scripts/update-shell-deps.sh

# Locks versions in vendor.lock
vendor/zim @ commit abc123
vendor/powerlevel10k @ v1.19.0
```

### Option B: Minimal Zsh Setup (No Framework)
Replace Zim with handcrafted minimal setup:
- Custom prompt (or vendor just P10k)
- Essential plugins only: autosuggestions, syntax-highlighting, history-substring-search
- All vendored in `shell/plugins/`

**Benefits**:
- Faster shell startup
- Complete control over code
- Easier to debug
- No runtime downloads

**Implementation**:
1. Decision: Vendor Zim or go frameworkless?
2. Create `scripts/vendor-deps.sh` to download & lock dependencies
3. Update `.zshrc` to source from `vendor/` instead of downloading
4. Add `vendor.lock` to track versions
5. Document update process

**Priority**: High - Improves portability and reliability

---

## 3. Global Keyboard Shortcut Reference & Conflict Detection

**Concept**: Centralized registry of all keyboard shortcuts across all tools, with automatic conflict detection.

**Problem**:
- Karabiner remaps keys
- Ghostty has terminal shortcuts
- Tmux has prefix bindings
- Neovim has mappings
- Raycast has global shortcuts
- macOS system shortcuts
- No way to know if shortcuts conflict!

**Solution**: Create `docs/keyboard-map.yaml` that aggregates all shortcuts.

**Features**:
- Parse configs to extract keybindings automatically
- Visual keyboard layout showing what each key does
- Conflict detection: "Warning: Cmd+K mapped in both Ghostty and Raycast"
- Interactive HTML reference (generated from YAML)
- Suggest alternative keybindings when conflicts found
- Filter by context (tmux mode, vim mode, normal, etc.)

**Example Format**:
```yaml
shortcuts:
  - key: "Cmd+T"
    context: "global"
    tool: "Ghostty"
    action: "New tab"

  - key: "Ctrl+Space"
    context: "tmux"
    tool: "tmux"
    action: "Open custom menu"
    defined_in: "config/tmux/tmux.conf:80"

  - key: "Ctrl+Space"
    context: "global"
    tool: "Raycast"
    action: "Open Raycast"
    conflict: true
    conflicts_with: ["tmux"]
```

**Implementation**:
1. Parser scripts for each tool's config format
2. `scripts/generate-keyboard-map.sh` → builds YAML
3. `scripts/check-keyboard-conflicts.sh` → validation
4. `scripts/generate-keyboard-html.sh` → interactive reference
5. Pre-commit hook to validate no new conflicts

**Tech Stack**:
- Parsers: Shell scripts + awk/sed for config parsing
- HTML generator: Python with Jinja2 or simple bash + HTML template
- Visual: keyboard-layout.js or custom CSS grid

**Priority**: High - Prevents frustrating conflicts, great for onboarding

---

## 4. Branding & Marketing

**Concept**: Turn this dotfiles repo into a showcase-worthy open-source project.

**Components**:

### Project Name & Identity
- Choose a memorable name (currently just "dotfiles")
- Ideas: "HomeBase", "DevKit", "Forge", "Atelier", "Studio"
- Logo/icon for README
- Consistent color scheme in docs

### Professional README
- Screenshots of terminal setup
- GIF demos of key features (fresh command, theme switching, etc.)
- Feature comparison matrix vs other popular dotfiles
- Testimonials or user count

### Documentation Website
- GitHub Pages site with beautiful docs
- Searchable documentation
- Video tutorials
- FAQ section

### Marketing Materials
- Blog post: "Building a Self-Contained Development Environment"
- Twitter/X thread showcasing features
- Hacker News post
- Reddit r/unixporn showcase
- YouTube setup walkthrough

### Badges & Stats
```markdown
![Homebrew Packages](https://img.shields.io/badge/homebrew-95_packages-orange)
![Shell](https://img.shields.io/badge/shell-zsh-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Last Update](https://img.shields.io/github/last-commit/user/dotfiles)
```

### Community Features
- CONTRIBUTING.md
- Issue templates for bugs/features
- Pull request template
- Code of conduct
- Changelog
- Release notes for major updates

**Priority**: Low-Medium - Great for portfolio, attracts contributors

---

## 5. Multi-Machine Profiles

**Concept**: Support different configurations for different machines (work, personal, minimal server, etc.)

**Use Cases**:
- Work laptop: Restricted tools, corporate settings, VPN configs
- Personal desktop: Full setup with gaming tools
- Remote server: Minimal CLI-only setup, no GUI apps
- iPad with Blink Shell: Ultra-minimal, no Homebrew

**Implementation**:

### Profile System
```
~/dotfiles/
├── profiles/
│   ├── full.yaml          # Everything (default)
│   ├── work.yaml          # Work restrictions
│   ├── minimal.yaml       # Server/remote
│   └── ipad.yaml          # iOS/Blink Shell
└── .dotfiles-profile      # Current machine profile
```

**Profile YAML** (`profiles/work.yaml`):
```yaml
name: "Work Machine"
description: "Corporate laptop with restrictions"

homebrew:
  exclude:
    - angband              # No games
    - yt-dlp              # Blocked by corp firewall
  include_all_others: true

configs:
  exclude:
    - config/obsidian     # Personal notes only

shell:
  aliases:
    work_vpn: "sudo openconnect vpn.corp.com"

environment:
  WORK_MODE: "true"
  CORP_PROXY: "http://proxy.corp.com:8080"

secrets:
  bitwarden_collection: "Work"

git:
  user_email: "name@workcorp.com"
  signing_key: "work-gpg-key-id"
```

**Usage**:
```bash
# Initial setup
./install.sh --profile work

# Switch profiles
./scripts/switch-profile.sh minimal

# Override for current session
DOTFILES_PROFILE=work ./scripts/setup_symlinks.sh
```

**Features**:
- Profile inheritance (work extends full)
- Per-profile package lists
- Conditional config sections
- Profile-specific secrets in separate Bitwarden collections
- Validation: "These packages required by profile are not installed"

**Priority**: High - Essential for multi-machine users

---

## 6. Automated Backup & Rollback System

**Concept**: Before any changes, create restorable snapshots with easy rollback.

**Features**:
- Pre-install backup of existing configs
- Snapshot before major changes
- Named restore points
- Diff between current and backup
- Automated weekly backups

**Implementation**:

### Backup System
```bash
# Create backup
./scripts/backup.sh --name "before-theme-change"

# List backups
./scripts/backup.sh --list
# Output:
# before-theme-change    2025-10-16 20:30
# fresh-install-backup   2025-10-15 10:00
# auto-weekly-2025-10-10 2025-10-10 00:00

# Restore backup
./scripts/restore.sh before-theme-change

# Diff current vs backup
./scripts/backup.sh --diff before-theme-change
```

### Backup Storage
```
~/.dotfiles-backups/
├── before-theme-change-20251016/
│   ├── manifest.json     # What was backed up
│   ├── .zshrc
│   ├── config/
│   └── metadata.json     # Timestamp, machine info
├── fresh-install-backup-20251015/
└── auto-weekly-2025-10-10/
```

**Advanced Features**:
- Incremental backups (only changed files)
- Compression for old backups
- Cloud sync option (Dropbox, iCloud, Syncthing)
- Backup verification (checksum)
- Backup retention policy (keep last 10, auto-delete old)

**Integration**:
- `install.sh` creates backup automatically
- Git hooks: backup before major commits
- Cron job for weekly auto-backup

**Priority**: High - Safety net for experimentation

---

## 7. Dotfiles CLI Tool

**Concept**: Unified command-line interface for all dotfiles operations.

**Current State**: Multiple scripts (install.sh, setup_symlinks.sh, macos-defaults.sh, etc.)

**Proposed**: Single `dot` command with subcommands

**Usage**:
```bash
dot install              # Fresh install
dot install --profile work

dot update               # Update all packages & dotfiles
dot update brew          # Update only Homebrew
dot update shell-deps    # Update Zim, P10k, etc.

dot sync                 # Pull latest from git, apply changes
dot sync --dry-run       # Show what would change

dot backup create        # Create backup
dot backup list          # List backups
dot backup restore <name>

dot theme list           # List available themes
dot theme apply mocha    # Apply theme
dot theme preview nord   # Preview without applying

dot profile current      # Show current profile
dot profile switch work  # Switch profile

dot health               # Run health checks
dot conflicts            # Check keyboard conflicts
dot validate             # Validate all configs

dot clean                # Remove backups, caches
dot info                 # Show dotfiles status, stats
```

**Implementation**:

### Structure
```
scripts/
├── dot                  # Main CLI (router to subcommands)
├── commands/
│   ├── install.sh
│   ├── update.sh
│   ├── sync.sh
│   ├── backup.sh
│   ├── theme.sh
│   ├── profile.sh
│   ├── health.sh
│   └── clean.sh
└── lib/
    ├── utils.sh         # Shared functions
    ├── colors.sh        # Output formatting
    └── validation.sh    # Config validation
```

**Features**:
- Bash completion for `dot` command
- Colorized output with spinners/progress bars
- `--verbose` and `--quiet` flags
- `--dry-run` for preview mode
- Man page: `man dot`
- Help text: `dot help <subcommand>`

**Tech Stack**:
- Pure bash for portability
- Use `getopts` for flag parsing
- Chalk/gum for beautiful output (optional)

**Priority**: Medium - Improves UX significantly

---

## 8. Package Drift Detection & Reconciliation

**Concept**: Detect when installed packages differ from Brewfile/package lists, suggest reconciliation.

**Problem**:
```bash
# You install something quickly
brew install some-tool

# Now your system != dotfiles
# Brewfile is out of date
```

**Solution**: Automated drift detection

**Features**:
- Compare installed vs declared packages
- Show packages installed but not in Brewfile
- Show packages in Brewfile but not installed
- Suggest adding to Brewfile or uninstalling
- Check all package managers: brew, npm, cargo, uv

**Implementation**:

```bash
./scripts/check-drift.sh

# Output:
# 📦 Package Drift Report
#
# ⚠️ Installed but not in dotfiles:
#   brew:   some-tool, another-package
#   npm:    random-cli
#   cargo:  experimental-tool
#
# 💡 Actions:
#   1. Add to dotfiles:  ./scripts/reconcile.sh --add
#   2. Remove packages:  ./scripts/reconcile.sh --remove
#   3. Ignore (add to .drift-ignore):  ./scripts/reconcile.sh --ignore
```

**Reconcile Options**:
```bash
# Interactive mode - choose what to do with each package
./scripts/reconcile.sh --interactive

# Auto-add everything to Brewfile
./scripts/reconcile.sh --add-all

# Auto-remove packages not in Brewfile
./scripts/reconcile.sh --remove-all  # dangerous!
```

**Drift Ignore File** (`.drift-ignore`):
```
# Temporary tools, don't track
npm:test-package
cargo:quick-experiment

# Work-specific tools
brew:corporate-vpn
```

**Automation**:
- Pre-commit hook: warn if drift detected
- Weekly cron: email drift report
- CI/CD: fail if drift in dotfiles repo

**Priority**: Medium-High - Keeps dotfiles accurate

---

## 9. Configuration Validation & Health Checks

**Concept**: Automated testing and validation of dotfiles before applying.

**Checks**:

### Syntax Validation
- Shell scripts: `shellcheck` on all .sh files
- YAML files: `yamllint` on configs
- JSON: `jq` validation
- Git configs: `git config --list` validation
- Symlink targets exist

### Integration Tests
- Does shell source without errors?
- Are all PATH entries valid directories?
- Do all aliases work?
- Can all completions load?
- Do all sourced external files exist?

### Security Checks
- No secrets in tracked files
- File permissions correct (SSH config 600, etc.)
- No world-writable files
- SSH keys not in repo
- API keys not in env files

### Compatibility Checks
- macOS version compatibility
- Required binaries exist (brew, git, etc.)
- Minimum versions met (zsh >= 5.8, etc.)
- Disk space sufficient

### Performance Checks
- Shell startup time < 500ms
- Config files not too large
- No circular symlinks

**Implementation**:

```bash
./scripts/health-check.sh

# Output:
# 🏥 Dotfiles Health Check
#
# ✅ Syntax Validation (5/5 passed)
# ✅ Integration Tests (12/12 passed)
# ⚠️  Security Checks (4/5 passed)
#     - Warning: .zshenv is world-readable
# ✅ Compatibility Checks (8/8 passed)
# ✅ Performance Checks (3/3 passed)
#
# Overall: 32/33 checks passed
#
# 💡 Recommendations:
#   - Fix: chmod 644 ~/.zshenv
```

**CI Integration**:
```yaml
# .github/workflows/validate.yml
name: Validate Dotfiles
on: [push, pull_request]
jobs:
  validate:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Health Check
        run: ./scripts/health-check.sh
      - name: Shellcheck
        run: shellcheck scripts/*.sh
```

**Priority**: High - Prevents broken configs from being committed

---

## 10. Documentation Generator from Comments

**Concept**: Auto-generate documentation from comments in config files.

**Problem**: Configs have inline comments explaining settings, but no unified docs.

**Solution**: Extract comments and generate comprehensive documentation.

**Example**:

```bash
# In .zshrc:
# === History Configuration ===
# Keep 10 million commands in history for searchability
HISTSIZE=10000000

# Share history across all terminal sessions in real-time
# This allows commands from one terminal to be available in others
setopt SHARE_HISTORY
```

**Generated Doc** (`docs/generated/shell-config.md`):
```markdown
# Shell Configuration

## History Configuration

Keep 10 million commands in history for searchability.

**Setting**: `HISTSIZE=10000000`
**File**: `shell/.zshrc:28`

---

Share history across all terminal sessions in real-time.
This allows commands from one terminal to be available in others.

**Setting**: `setopt SHARE_HISTORY`
**File**: `shell/.zshrc:31`
```

**Features**:
- Parse comments from all config files
- Group by section headers
- Cross-reference settings
- Generate markdown, HTML, or man pages
- Include code examples
- Link to original file:line

**Implementation**:

```bash
# Generate docs
./scripts/generate-docs.sh

# Watch mode - regenerate on changes
./scripts/generate-docs.sh --watch

# Format options
./scripts/generate-docs.sh --format html
./scripts/generate-docs.sh --format man
```

**Special Comment Syntax**:
```bash
#@ title: History Configuration
#@ category: Shell
#@ importance: high
#@ see-also: shell-plugins.md
#
#@ This section configures zsh history behavior.
#@ History is shared across all sessions and persisted
#@ to disk immediately.

HISTSIZE=10000000
```

**Priority**: Low-Medium - Nice for maintainability

---

## 11. Dotfiles Analytics & Insights

**Concept**: Collect anonymous usage data to optimize your dotfiles.

**Metrics to Track**:
- Shell startup time over time
- Most-used aliases/functions
- Most-launched commands
- Plugin load times
- Package update frequency
- Config change frequency
- Backup creation count
- Failed installations

**Dashboard**:
```bash
./scripts/stats.sh

# Output:
# 📊 Dotfiles Analytics (Last 30 Days)
#
# ⚡ Performance:
#   Shell Startup: 287ms (avg), 312ms (max)
#   Slowest Plugin: powerlevel10k (124ms)
#
# 🔥 Most Used:
#   Commands: git (2,847), ls (1,203), cd (892)
#   Aliases:  gs (534), gp (289), d (203)
#   Functions: tb (156), cdx (89), glowf (34)
#
# 📦 Package Health:
#   Outdated: 12 packages
#   Last Updated: 3 days ago
#   Drift Detected: 2 packages
#
# 🔄 Activity:
#   Config Changes: 23 files
#   Backups Created: 4
#   Themes Applied: 2
```

**Privacy**:
- All data stored locally (no telemetry)
- Opt-in analytics
- Sanitize sensitive data (no command args)

**Implementation**:
- Shell hook to log commands: `preexec` function
- Startup time tracking in `.zshrc`
- Store in `~/.dotfiles-analytics.db` (SQLite)
- Query with `scripts/stats.sh`

**Advanced Features**:
- Time-series graphs (using termgraph)
- Recommendations: "You use `git status` often, consider aliasing to `gs`"
- Efficiency score: "Your dotfiles health: 87/100"

**Priority**: Low - Fun but not essential

---

## 12. AI-Powered Dotfile Optimization (WILD)

**Concept**: Use AI to analyze your dotfiles and suggest improvements.

**Features**:

### Smart Suggestions
```bash
./scripts/ai-optimize.sh

# Output:
# 🤖 AI Analysis Complete
#
# 💡 Suggestions:
#
# 1. Reduce shell startup time (Current: 287ms)
#    - Move pyenv init to lazy-load (saves ~80ms)
#    - Defer p10k instant prompt (saves ~40ms)
#    - Estimated new startup: 167ms
#
# 2. Unused packages detected:
#    - `angband` - not launched in 90 days
#    - `yt-dlp` - never used
#    - Potential savings: 45MB disk space
#
# 3. Security improvements:
#    - SSH config world-readable (fix: chmod 600)
#    - Bitwarden session expired (last unlock: 7 days ago)
#
# 4. Recommended packages based on your usage:
#    - `fzf-git` - you use git 2,847 times/month
#    - `zoxide` - you use cd 892 times/month
#    - `lazydocker` - you use docker 234 times/month
```

### Learning Aliases
AI observes your command patterns and suggests aliases:
```bash
# You type this often:
git commit -m "..." && git push

# AI suggests:
alias gcp='git commit -m "$1" && git push'
```

### Configuration Validation
- AI reviews configs for best practices
- Suggests improvements based on your usage patterns
- Compares to popular dotfiles repos
- Finds outdated patterns

**Implementation**:
- Local LLM (Ollama) or API (OpenAI, Anthropic)
- Analyze shell history, config files
- Generate structured suggestions
- Apply suggestions interactively

**Tech Stack**:
- Python script using OpenAI API or local LLM
- Shell history parser
- Config file analyzer
- Interactive TUI for applying suggestions

**Priority**: Wild - Very experimental, fun hackathon project

---

## 13. Collaborative Dotfiles Marketplace (WILD)

**Concept**: Share and discover dotfiles modules/plugins from community.

**Features**:

### Module System
```bash
# Browse marketplace
dot marketplace search "tmux"

# Output:
# 🏪 Dotfiles Marketplace
#
# tmux-session-wizard    ⭐ 1.2k   Fast tmux session management
# tmux-plugin-pack       ⭐ 856    Curated tmux plugins
# tmux-catppuccin-theme  ⭐ 643    Beautiful tmux theme

# Install module
dot marketplace install tmux-session-wizard

# This adds module to your dotfiles and applies it
```

### Module Types
- **Themes**: Color schemes that work across tools
- **Config Bundles**: Pre-configured tool setups (neovim config, tmux config)
- **Shell Plugins**: Aliases, functions, completions
- **Workflows**: Fresh-like session setups for specific tasks
- **Profiles**: Complete machine profiles (web dev, data science, etc.)

### Publishing
```bash
# Package your tmux config as module
dot marketplace package \
  --name "my-tmux-setup" \
  --description "Tmux config with vim bindings" \
  --files "config/tmux/*" \
  --category "tmux"

# Publish to marketplace
dot marketplace publish my-tmux-setup
```

### Module Manifest** (`module.yaml`):
```yaml
name: "tmux-session-wizard"
version: "1.0.0"
author: "username"
description: "Fast tmux session management with fzf"
category: "tmux"

dependencies:
  homebrew:
    - tmux
    - fzf

files:
  - src: "scripts/session-wizard.sh"
    dest: "scripts/session-wizard.sh"
  - src: "config/tmux-wizard.conf"
    dest: "config/tmux/wizard.conf"

install:
  - "source config/tmux/wizard.conf in .tmux.conf"

uninstall:
  - "remove source line from .tmux.conf"
```

### Social Features
- Star/fork modules
- Reviews and ratings
- Top modules by category
- Trending this week
- User profiles showing their stack

**Implementation**:
- Central registry (GitHub repo or dedicated site)
- Module validation before publishing
- Semantic versioning
- Update notifications

**Tech Stack**:
- Registry: GitHub with API or custom backend
- CLI: Python or Go for module management
- Web: Next.js for marketplace website

**Priority**: Wild - Requires community, but could be huge

---

## 14. Time-Travel Debugging for Config Changes (WILD)

**Concept**: Record every config change with full context, replay your setup at any point in time.

**Features**:

### Record Everything
- Every file edit with timestamp
- Every package install/uninstall
- Every setting change
- Commands that triggered changes
- System state (OS version, machine, etc.)

### Time-Travel Commands
```bash
# Show what changed yesterday
dot history --since "yesterday"

# Output:
# 📅 Changes on 2025-10-16
#
# 14:32:15 - Installed package: brew install neovim
# 14:35:02 - Modified: config/nvim/init.lua (+23 -5 lines)
# 16:21:43 - Applied theme: catppuccin-mocha
#            - Updated: config/ghostty/config
#            - Updated: config/tmux/tmux.conf
#            - Updated: config/nvim/init.lua
# 20:15:33 - Created backup: before-experiment

# Replay setup from specific date
dot time-travel --to "2025-10-01"

# This rolls back ALL configs to Oct 1st state

# Compare two points in time
dot diff --from "2025-10-01" --to "2025-10-16"

# Bisect to find breaking change
dot bisect start
# Mark current state as bad
dot bisect bad
# Mark working state as good
dot bisect good 2025-10-01
# Binary search through changes to find culprit
```

### Change Attribution
Every change knows:
- What command triggered it
- What human or script made it
- Why (commit message, PR description)
- What else changed at the same time

### Visualization
```bash
dot timeline

# ASCII timeline:
#
# Oct 01  Oct 05  Oct 10  Oct 15  Oct 20
# ───┬──────┬──────┬──────┬──────┬────
#    │      │      │      │      │
#    │      │      │      │      └─ Theme: mocha
#    │      │      │      └─ Added: tmux
#    │      │      └─ Installed: 23 packages
#    │      └─ Profile: work
#    └─ Fresh install
```

### Event Sourcing
All changes stored as events:
```json
{
  "timestamp": "2025-10-16T20:15:33Z",
  "type": "file_modified",
  "file": "config/nvim/init.lua",
  "diff": "...",
  "trigger": "manual edit",
  "command": "nvim config/nvim/init.lua",
  "user": "bwl",
  "machine": "macbook-pro",
  "commit": "abc123"
}
```

**Implementation**:
- Git hooks to record all changes
- Event store (SQLite or JSON files)
- Replay engine to restore state
- Diff engine for comparisons

**Tech Stack**:
- Git for file history
- SQLite for event store
- Python for replay logic
- termgraph for visualization

**Priority**: Wild - Complex but incredibly powerful

---

## 15. Dotfiles as Infrastructure-as-Code (WILD)

**Concept**: Treat dotfiles like Terraform - declarative config with plan/apply workflow.

**Declarative Model**:

### Desired State File** (`dotfiles.tf` or `dotfiles.hcl`):
```hcl
machine "macbook-pro" {
  profile = "work"

  packages {
    homebrew = ["neovim", "tmux", "ripgrep"]
    npm      = ["@openai/codex"]
    cargo    = ["repo2prompt"]
  }

  configs {
    shell {
      theme    = "catppuccin-mocha"
      plugins  = ["zim", "powerlevel10k"]
      history_size = 10000000
    }

    git {
      user_name  = "Your Name"
      user_email = "you@work.com"
      signing_key = var.gpg_key_id
    }

    tmux {
      prefix_key = "C-Space"
      mouse      = true
      plugins    = ["tpm", "catppuccin"]
    }
  }

  symlinks {
    link {
      source = "~/dotfiles/shell/.zshrc"
      target = "~/.zshrc"
    }
  }

  macos_defaults {
    dock {
      autohide = true
      position = "bottom"
    }

    finder {
      show_hidden_files = true
      show_path_bar     = true
    }
  }
}
```

### Terraform-like Workflow
```bash
# Plan changes (dry-run)
dot plan

# Output:
# 🔍 Plan: 5 changes
#
# + Install packages:
#     brew: neovim, tmux
#     npm:  @openai/codex
#
# ~ Update configs:
#     config/tmux/tmux.conf: mouse off → on
#
# - Remove packages:
#     brew: unused-package
#
# To apply: dot apply

# Apply changes
dot apply

# Output:
# ✓ Installing neovim... done
# ✓ Installing tmux... done
# ✓ Updating tmux.conf... done
#
# Apply complete! 5 changes applied.

# Show current state
dot show

# Drift detection
dot plan
# Output: No changes needed. Dotfiles are up to date.
```

### State Management
- State file tracks actual system state
- Compare desired vs actual state
- Only apply differences
- Rollback on failure

### Modules & Reusability
```hcl
# Use modules
module "web_dev_tools" {
  source = "./modules/web-dev"

  node_version = "20"
  include_docker = true
}

module "ai_tools" {
  source = "github.com/user/dotfiles-modules//ai-tools"
}
```

### Variables & Secrets
```hcl
variable "work_email" {
  type = string
}

variable "gpg_key_id" {
  type      = string
  sensitive = true
}

# Use in config
git {
  user_email  = var.work_email
  signing_key = var.gpg_key_id
}
```

**Implementation**:
- HCL parser (use Terraform's libraries or custom)
- State file (JSON/SQLite)
- Diff engine
- Apply engine with rollback
- Module system

**Tech Stack**:
- Go (like Terraform) or Python
- HCL parsing library
- State management
- Execution engine

**Priority**: Wild - Massive undertaking, but ultimate dotfiles management

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
**Priority: High-value, low-effort**
1. Self-Contained Zsh Dependencies (#2)
2. Multi-Machine Profiles (#5)
3. Automated Backup System (#6)
4. Health Checks (#9)

### Phase 2: Developer Experience (Weeks 3-4)
**Priority: Improve daily usage**
5. Dotfiles CLI Tool (#7)
6. Package Drift Detection (#8)
7. Keyboard Shortcut Reference (#3)

### Phase 3: Polish (Weeks 5-6)
**Priority: Nice-to-haves**
8. Global Theme Config (#1)
9. Documentation Generator (#10)
10. Branding & Marketing (#4)

### Phase 4: Experimental (Future)
**Priority: Exploration & fun**
11. Dotfiles Analytics (#11)
12. AI-Powered Optimization (#12)

### Phase 5: Moonshots (If successful)
**Priority: Dream big**
13. Collaborative Marketplace (#13)
14. Time-Travel Debugging (#14)
15. Infrastructure-as-Code (#15)

---

## Decision Matrix

| Idea | Impact | Effort | Priority | Fun Factor |
|------|--------|--------|----------|------------|
| 1. Global Theme | Medium | Medium | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| 2. Self-Contained Deps | High | Medium | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| 3. Keyboard Map | High | Medium | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| 4. Branding | Low | Medium | ⭐⭐ | ⭐⭐⭐ |
| 5. Multi-Machine | High | Medium | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| 6. Backup System | High | Low | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| 7. CLI Tool | High | High | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| 8. Drift Detection | Medium | Low | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| 9. Health Checks | High | Low | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| 10. Doc Generator | Low | Medium | ⭐⭐ | ⭐⭐ |
| 11. Analytics | Low | Medium | ⭐⭐ | ⭐⭐⭐⭐ |
| 12. AI Optimization | Medium | High | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 13. Marketplace | High | Very High | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 14. Time-Travel | Medium | Very High | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 15. IaC Model | High | Very High | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## Quick Wins (Can Implement Today)

1. **Drift Detection Script** (~1 hour)
   - Compare `brew list` with Brewfile
   - Simple bash script

2. **Backup Script** (~2 hours)
   - Copy configs to timestamped directory
   - List/restore functionality

3. **Health Check** (~2 hours)
   - Shellcheck all scripts
   - Validate symlinks exist
   - Check for secrets

4. **Keyboard Shortcuts Markdown** (~3 hours)
   - Manual doc listing shortcuts
   - Can automate later

---

## Notes

### Related Projects to Study
- **dotbot** - Dotfile installer
- **chezmoi** - Dotfile manager with templating
- **yadm** - Yet Another Dotfiles Manager
- **GNU Stow** - Symlink farm manager
- **Ansible** - Infrastructure automation (for IaC inspiration)
- **Terraform** - Infrastructure as code (for declarative model)

### Technologies to Explore
- **HCL** - HashiCorp Configuration Language
- **Nix** - Declarative package management
- **Home Manager** - Nix-based dotfile management
- **direnv** - Per-directory environments

### Open Questions
1. Should we vendor Zim or go frameworkless?
2. What's the best profile format - YAML, TOML, HCL?
3. How to handle secrets in profiles securely?
4. Should `dot` CLI be bash or Python/Go?
5. Marketplace: centralized or distributed (git submodules)?

---

**Next Steps**: Pick 1-2 quick wins to implement this week, then tackle Phase 1 systematically.
