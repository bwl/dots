# bwl/dots

Personal macOS development environment with automated setup and Bitwarden secret management.

**Contents:**
- 55 Homebrew packages
- 10 Mac App Store apps
- Shell configs (zsh + Powerlevel10k)
- Development tools (neovim, rust, bun, python/uv)
- Application configs (ghostty, tmux, karabiner, VS Code)
- System preferences automation

---

## Quick Start

### Fresh Install

```bash
git clone https://github.com/bwl/dots.git ~/dotfiles
cd ~/dotfiles
./install.sh
```

The script will:
- Install Xcode Command Line Tools
- Install Homebrew and all packages
- Install Mac App Store apps
- Create symlinks

After installation:
```bash
# Setup Bitwarden
bw login
export BW_SESSION="$(bw unlock --raw)"

# Reload shell
exec zsh

# Configure theme
p10k configure
```

### Existing System

Update symlinks only:
```bash
cd ~/dotfiles
./scripts/setup_symlinks.sh --force
```

---

## Repository Structure

```
dotfiles/
├── shell/          # .zshrc, .zshenv, .p10k.zsh
├── config/         # Application configs (nvim, tmux, ghostty, etc.)
├── git/            # .gitconfig, .gitignore_global
├── ssh/            # SSH client config
├── vscode/         # VS Code settings
├── bin/            # User scripts
├── scripts/        # Setup and utility scripts
├── Brewfile        # Homebrew packages
├── Masfile         # Mac App Store apps
├── npm-globals.txt # npm packages
├── cargo-installs.txt # Rust binaries
├── uv-tools.txt    # Python tools
└── install.sh      # Bootstrap script
```

---

## Package Management

### Update Package Lists

After installing new packages:
```bash
cd ~/dotfiles

# Homebrew
brew bundle dump --force --file=Brewfile

# Mac App Store
mas list > Masfile

# npm globals
npm list -g --depth=0 | tail -n +2 | sed 's/^[├└]─* //' > npm-globals.txt

# Cargo
ls ~/.cargo/bin/ | grep -v "^cargo\|^rust" > cargo-installs.txt

# uv tools
uv tool list | grep -E "^\S" | awk '{print $1}' > uv-tools.txt

git add . && git commit -m "Update packages"
```

### Restore Packages

```bash
# Homebrew (automatic via install.sh)
brew bundle install

# npm
cat npm-globals.txt | grep -v "^#" | xargs npm install -g

# Cargo
cat cargo-installs.txt | grep -v "^#" | xargs cargo install

# uv
cat uv-tools.txt | grep -v "^#" | xargs -I {} uv tool install {}

# VS Code extensions
cat vscode-extensions.txt | xargs -L 1 code --install-extension
```

---

## macOS System Preferences

Apply automated system preferences:
```bash
cd ~/dotfiles
./macos-defaults.sh
```

Configures:
- Dock auto-hide and position
- Finder hidden files and extensions
- Fast key repeat
- Screenshot location
- Trackpad tap-to-click
- Menu bar items

---

## Security

### Never Commit

- SSH private keys
- API keys/tokens
- `.env` files
- Bitwarden sessions

All sensitive files are excluded via `.gitignore`.

### Bitwarden Secret Management

Store secrets:
```bash
bw login
bw create item --type login --name "API Key Name" --notes "your-secret-here"
```

Reference in shell config:
```bash
# In .zshenv
export MY_API_KEY="$(bw get notes 'API Key Name')"
```

Auto-unlock in `.zshrc`:
```bash
if ! bw status | grep -q '"status":"unlocked"'; then
  export BW_SESSION="$(bw unlock --raw)"
fi
```

---

## Maintenance

### Update Everything

```bash
# Update Homebrew packages
brew update && brew upgrade

# Pull latest dotfiles
cd ~/dotfiles
git pull

# Reinstall if needed
brew bundle install
./scripts/setup_symlinks.sh --force
```

### Cleanup

```bash
# Remove unused Homebrew dependencies
brew autoremove

# Clean old versions
brew cleanup

# Check for issues
brew doctor
```

---

## Troubleshooting

**Symlinks not working:**
```bash
ls -la ~/.zshrc  # Check symlink
cd ~/dotfiles && ./scripts/setup_symlinks.sh --force
```

**Homebrew fails:**
```bash
brew update
brew doctor
```

**Bitwarden unlock:**
```bash
bw login
bw unlock  # Copy session key
bw status  # Verify
```

**Config not loading:**
```bash
source ~/.zshrc  # Check for errors
ls -la ~/.config/appname  # Verify symlink
```

**Restore from backup:**
```bash
ls ~/dotfiles_backup_*
cp ~/dotfiles_backup_YYYYMMDD_HHMMSS/.zshrc ~/
```

---

## Additional Documentation

- [APPLICATIONS.md](APPLICATIONS.md) - Application-specific configs
- [CLAUDE.md](CLAUDE.md) - AI assistant instructions

---

## License

MIT
