# 🏠 Dotfiles

My personal macOS development environment configuration, managed through version control and automated with Bitwarden CLI for secure secret management.

**Stats:**
- **95 Homebrew formulae** (curated from 296 - 68% reduction!)
- **10 Mac App Store apps**
- **8 npm global packages** + **5 cargo installs** + **3 uv tools**
- **108 VS Code extensions** + application configs
- **macOS system preferences** automation
- **Secure secret management** via Bitwarden CLI
- **Automated setup** for fresh installs

---

## 📋 Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Fresh Install](#fresh-install)
- [Existing System](#existing-system)
- [What's Included](#whats-included)
- [Application Preferences](#application-preferences)
- [Security](#security)
- [Maintenance](#maintenance)
- [Troubleshooting](#troubleshooting)

---

## 🎯 Overview

This repository contains:
- **Shell configurations** (zsh with Powerlevel10k)
- **Development tools** (rust, bun, uv, pyenv, neovim)
- **CLI utilities** (bat, fd, fzf, ripgrep, eza, yazi)
- **Git configuration** and workflow tools
- **Application configs** (ghostty, nvim, karabiner, tmux)
- **Package manager inventories** (npm, cargo, uv tools)
- **Automated setup scripts** for reproducible environments

---

## 🚀 Quick Start

### For New Machines

```bash
# 1. Clone this repository
git clone https://github.com/yourusername/dotfiles.git ~/dotfiles
cd ~/dotfiles

# 2. Run the bootstrap script
./install.sh

# 3. Follow the prompts for:
#    - Homebrew package installation
#    - Mac App Store apps
#    - Symlink creation
#    - Bitwarden setup
```

### For Existing Dotfiles

```bash
# Just update symlinks
cd ~/dotfiles
./scripts/setup_symlinks.sh --force
```

---

## 🆕 Fresh Install

### Prerequisites

1. **macOS** (tested on macOS Sonoma)
2. **Xcode Command Line Tools** (installed automatically by install.sh)
3. **Bitwarden account** for secret management

### Installation Steps

1. **Clone the repository:**
   ```bash
   git clone <your-repo-url> ~/dotfiles
   cd ~/dotfiles
   ```

2. **Run the bootstrap script:**
   ```bash
   ./install.sh
   ```

   This will:
   - Install Xcode Command Line Tools (if needed)
   - Install Homebrew
   - Install all packages from Brewfile (~15-30 minutes)
   - Install Mac App Store apps (requires App Store login)
   - Setup Oh-My-Zsh and Powerlevel10k
   - Create symlinks to your home directory

3. **Setup Bitwarden CLI:**
   ```bash
   bw login
   export BW_SESSION="$(bw unlock --raw)"
   ```

4. **Migrate secrets to Bitwarden** (see [Security](#security) section)

5. **Reload your shell:**
   ```bash
   exec zsh
   ```

6. **Configure Powerlevel10k:**
   ```bash
   p10k configure
   ```

---

## 🔧 Existing System

### Update Packages

```bash
cd ~/dotfiles

# Update Brewfile with currently installed packages
brew bundle dump --force --file=Brewfile

# Update Masfile with currently installed apps
mas list > Masfile

# Update npm globals
npm list -g --depth=0 | tail -n +2 | sed 's/^[├└]─* //' > npm-globals.txt

# Update cargo installs
ls ~/.cargo/bin/ | grep -v "^cargo\|^rust" > cargo-installs.txt

# Update uv tools
uv tool list | grep -E "^\S" | awk '{print $1}' > uv-tools.txt

# Commit changes
git add Brewfile Masfile npm-globals.txt cargo-installs.txt uv-tools.txt
git commit -m "Update package lists $(date +%Y-%m-%d)"
```

### Add New Configuration

```bash
cd ~/dotfiles

# Copy new config to dotfiles
cp -r ~/.config/newapp config/

# Add to git
git add config/newapp
git commit -m "Add newapp configuration"

# Push to remote
git push
```

### Apply Changes on Another Machine

```bash
cd ~/dotfiles
git pull

# Reinstall packages
brew bundle install

# Update symlinks
./scripts/setup_symlinks.sh --force
```

---

## 📦 What's Included

### Development Tools

- **Languages**: Rust, JavaScript/TypeScript (Bun), Python (uv + pyenv)
- **Editors**: Neovim, msedit, VS Code (settings)
- **AI Tools**: qwen-code, opencode, specify, crush

### CLI Utilities

| Tool | Purpose | Replaces |
|------|---------|----------|
| `bat` | Syntax-highlighted cat | cat |
| `fd` | Fast file finder | find |
| `fzf` | Fuzzy finder | - |
| `ripgrep` | Fast grep | grep, ag |
| `eza` | Modern ls | ls |
| `yazi` | File manager | - |
| `jq` | JSON processor | - |
| `sd` | Sed alternative | sed |

### Git Tools

- `gh` - GitHub CLI
- `lazygit` - Terminal UI for git
- `tig` - Text-mode git interface
- `git-lfs` - Large File Storage
- `git-filter-repo` - Repository history rewriting

### System Monitoring

- `htop` - Process viewer
- `mactop` - macOS activity monitor
- `ncdu` - Disk usage analyzer
- `procs` - Modern ps

### Configuration Files

```
~/dotfiles/
├── config/
│   ├── ghostty/      # Terminal emulator
│   ├── nvim/         # Neovim config
│   ├── karabiner/    # Keyboard customization
│   ├── tmux/         # Terminal multiplexer
│   ├── gh/           # GitHub CLI
│   ├── fd/           # fd config
│   └── yazi/         # File manager
├── shell/
│   ├── .zshrc        # Main shell config
│   ├── .zprofile     # Login shell
│   ├── .zshenv       # Environment variables
│   └── .p10k.zsh     # Powerlevel10k theme
├── git/
│   ├── .gitconfig
│   └── .gitignore_global
├── ssh/
│   └── config        # SSH client config
└── vscode/
    └── settings.json # VS Code preferences
```

### Package Manager Inventories

Beyond Homebrew, this repo tracks globally installed packages from various package managers:

**npm-globals.txt** (8 packages)
```bash
# AI/LLM CLIs
@github/copilot       # GitHub Copilot CLI
@google/gemini-cli    # Google Gemini CLI
@google/jules         # Jules AI assistant
@openai/codex         # OpenAI Codex CLI
@sourcegraph/amp      # Sourcegraph Amp

# Development Tools
happy-coder           # Coding assistant
vscode-langservers-extracted  # VS Code language servers
@randomlabs/slatecli  # Slate CLI

# Restore: cat npm-globals.txt | grep -v "^#" | xargs npm install -g
```

**cargo-installs.txt** (5 packages)
```bash
basalt-tui            # TUI framework
bevy_debugger_mcp     # Bevy game engine debugger
cargo-watch           # Auto-rebuild on file changes
repo2prompt           # Convert repos to LLM prompts
ttyper                # Terminal typing test

# Restore: cat cargo-installs.txt | grep -v "^#" | xargs cargo install
```

**uv-tools.txt** (3 packages)
```bash
craft-cli             # Craft CLI tool
posting               # API testing tool
writekit-cli          # Writing toolkit

# Restore: cat uv-tools.txt | grep -v "^#" | xargs -I {} uv tool install {}
```

---

## 🎨 Application Preferences

Beyond CLI tools, this repo manages GUI application settings and macOS system preferences.

### macOS System Preferences

**Automated via `macos-defaults.sh`:**
```bash
cd ~/dotfiles
./macos-defaults.sh
```

**Settings configured:**
- Dock: Auto-hide, position, no magnification
- Finder: Show hidden files, extensions, path bar
- Screenshots: Save to ~/Downloads as PNG
- Keyboard: Fast key repeat, disabled auto-correct
- Trackpad: Tap to click
- Menu Bar: Battery %, date/time
- System: Expanded panels, save to disk default

### Application Configs

**VS Code** (108 extensions)
```bash
# Restore extensions
cat vscode-extensions.txt | xargs -L 1 code --install-extension

# Settings symlinked automatically via setup_symlinks.sh
```

**Syncthing**
```bash
# Config tracked in app-configs/syncthing-config.xml
# Contains device IDs and folder sync settings
cp ~/dotfiles/app-configs/syncthing-config.xml \
  "$HOME/Library/Application Support/Syncthing/config.xml"
```

**Raycast** (41 extensions)
- Extension list tracked for reference
- Extensions sync via Raycast cloud account
- Config.json NOT tracked (contains access token)

**Obsidian**
- Vault locations documented in app-configs/obsidian-vaults.md
- Vault-specific settings sync with vault content
- Reference only, not automated restore

**See [APPLICATIONS.md](APPLICATIONS.md) for complete guide.**

---

## 🔐 Security

### ⚠️ NEVER Commit These:

- SSH private keys (`id_*`, `*.pem`, `*.key`)
- API keys or tokens
- `.env` files
- Passwords or secrets
- Bitwarden session keys

### Secret Management with Bitwarden

1. **Store secret in Bitwarden:**
   ```bash
   bw login
   bw create item --type login --name "API Key Name" --notes "your-secret-key"
   ```

2. **Reference in shell config:**
   ```bash
   # In ~/.zshenv (already in dotfiles/shell/.zshenv)
   export MY_API_KEY="$(bw get notes 'API Key Name')"
   ```

3. **Unlock Bitwarden on shell start:**
   ```bash
   # Add to ~/.zshrc (if not already there)
   if ! bw status | grep -q '"status":"unlocked"'; then
     export BW_SESSION="$(bw unlock --raw)"
   fi
   ```

### Current Secret to Migrate

Your `.zshenv` currently has:
```bash
export CLIFFY_OPENROUTER_API_KEY="sk-or-v1-fd211..."
```

**Action required:**
1. Store in Bitwarden: `bw create item --type login --name "CLIFFY OpenRouter API Key" --notes "sk-or-v1-fd211..."`
2. Update `.zshenv`: `export CLIFFY_OPENROUTER_API_KEY="$(bw get notes 'CLIFFY OpenRouter API Key')"`

---

## 🔄 Maintenance

### Update Everything

```bash
# Update Homebrew and packages
brew update && brew upgrade

# Update dotfiles repo
cd ~/dotfiles
git pull

# Regenerate Brewfile to capture new packages
brew bundle dump --force

# Regenerate Masfile
mas list > Masfile

# Commit updates
git add Brewfile Masfile
git commit -m "Update package lists $(date +%Y-%m-%d)"
git push
```

### Clean Up Homebrew

```bash
# Remove unused dependencies
brew autoremove

# Clean up old versions
brew cleanup

# Check for issues
brew doctor
```

### Backup Before Major Changes

```bash
# Create backup
cp -r ~/.config ~/.config.backup.$(date +%Y%m%d)
cp ~/.zshrc ~/.zshrc.backup.$(date +%Y%m%d)

# Or use Time Machine before running install.sh
```

---

## 🐛 Troubleshooting

### Symlinks Not Working

```bash
# Check if symlink exists and points correctly
ls -la ~/.zshrc

# Recreate symlinks with force
cd ~/dotfiles
./scripts/setup_symlinks.sh --force
```

### Homebrew Installation Fails

```bash
# Update Homebrew
brew update

# Try installing individually
brew install <package-name>

# Check for conflicts
brew doctor
```

### Bitwarden Can't Unlock

```bash
# Ensure you're logged in
bw login

# Get session key manually
bw unlock
# Then copy and export the session key shown

# Verify unlock status
bw status
```

### Shell Config Not Loading

```bash
# Check if symlink is correct
ls -la ~/.zshrc

# Source manually to see errors
source ~/.zshrc

# Check file permissions
chmod 644 ~/.zshrc
```

### App Not Finding Config

```bash
# Ensure config directory is symlinked
ls -la ~/.config/appname

# If not, recreate symlink
ln -sf ~/dotfiles/config/appname ~/.config/appname
```

### Restore from Backup

```bash
# If setup_symlinks.sh created backups
ls ~/dotfiles_backup_*

# Restore specific file
cp ~/dotfiles_backup_YYYYMMDD_HHMMSS/.zshrc ~/

# Or restore all
cp -r ~/dotfiles_backup_YYYYMMDD_HHMMSS/. ~/
```

---

## 📝 Notes

### Cleanup Performed

This setup represents a **68% reduction** from 296 to 95 Homebrew packages through careful curation:

- ✅ Removed unused terminal emulators (wezterm, kitty, rio, iterm2)
- ✅ Removed unused editors (helix, doom/emacs, zed)
- ✅ Removed redundant search tools (ag, GNU grep)
- ✅ Removed redundant file managers (kept only yazi)
- ✅ Cleaned up ~/.config/ directories

### Tools Kept By Design

- **daylight**: Dashboard sun tracker
- **msedit**: Retro but solid Microsoft terminal editor
- **opencode, crush, qwen-code, specify**: AI tooling evaluation
- **angband**: Active gaming
- **fortune**: Dashboard quotes
- **brew-explorer**: Homebrew TUI management

### Per-Machine Customization

For machine-specific configs (work vs personal), create:
```bash
# ~/.zshrc.local (not tracked in git)
# Add to .gitignore: *.local

# Then source in .zshrc:
[[ -f ~/.zshrc.local ]] && source ~/.zshrc.local
```

---

## 🤝 Contributing

This is a personal dotfiles repo, but feel free to:
- Fork for your own use
- Open issues for questions
- Submit PRs for improvements

---

## 📄 License

MIT License - Feel free to use and modify for your own dotfiles setup.

---

## 🔗 Resources

- [Homebrew](https://brew.sh/)
- [Bitwarden CLI](https://bitwarden.com/help/cli/)
- [Oh My Zsh](https://ohmyz.sh/)
- [Powerlevel10k](https://github.com/romkatv/powerlevel10k)
- [Neovim](https://neovim.io/)
- [Ghostty](https://ghostty.org/)

---

**Last Updated:** 2025-10-16
