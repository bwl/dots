# Application Configs

Configuration files and settings tracked in the repository.

## Shell Environment

### Zsh Configuration

Located in `shell/`

| File | Description | Symlinked To |
|------|-------------|--------------|
| .zshrc | Main shell configuration | ~/.zshrc |
| .zshenv | Environment variables | ~/.zshenv |
| .zprofile | Login shell configuration | ~/.zprofile |
| .p10k.zsh | Powerlevel10k theme config | ~/.p10k.zsh |

**Features:**
- Zim framework for plugin management
- Powerlevel10k theme
- 10M command history
- Custom functions: `tb()`, `git()`, `cdx()`, `glowf()`

## Text Editors

### Neovim

Located in `config/nvim/`

**Configuration:**
- LazyVim distribution
- LSP support for multiple languages
- Custom keybindings and plugins

**Files:**
- `init.lua` - Entry point
- `lua/config/` - Core configuration
- `lua/plugins/` - Plugin specifications
- `lazy-lock.json` - Plugin versions

### VS Code

Located in `vscode/`

**Files:**
- `settings.json` → `~/Library/Application Support/Code/User/settings.json`
- Extensions tracked in `vscode-extensions.txt`

## Terminal

### Ghostty

Located in `config/ghostty/`

| File | Description |
|------|-------------|
| config | Main terminal configuration |
| switch-shader.sh | Shader switching script |
| shaders/ | Visual shader effects (submodule) |

### Tmux

Located in `config/tmux/`

| File/Directory | Description |
|----------------|-------------|
| tmux.conf | Main tmux configuration |
| custom-menu.sh | Custom menu script |
| plugins/ | TPM plugins (submodules) |
| tmux-menus/ | Menu system (submodule) |

**Plugins (via TPM):**
- catppuccin/tmux - Theme
- tmux-battery - Battery status
- tmux-cpu - CPU/Memory usage
- tmux-fzf - Fuzzy finder integration
- tmux-harpoon - Quick navigation
- tmux-pop - Popup windows
- tmux-prefix-highlight - Prefix indicator
- tmux-sensible - Sensible defaults
- tmux-sidebar - File tree sidebar
- tmux-yank - Clipboard integration
- tpm - Plugin manager

### Tmuxifier

Located in `config/tmuxifier/`

**Layouts:**
- `retro-os.session.sh` - Main session layout
- `retro-os-main.window.sh` - Window configuration

## Development Tools

### Git

Located in `git/`

| File | Description | Symlinked To |
|------|-------------|--------------|
| .gitconfig | Git user config and aliases | ~/.gitconfig |
| .gitignore_global | Global ignore patterns | ~/.gitignore_global |

### SSH

Located in `ssh/`

| File | Description | Symlinked To |
|------|-------------|--------------|
| config | SSH client configuration | ~/.ssh/config |

**Note:** Private keys are NOT tracked (excluded via .gitignore)

## Tool Configurations

### GitHub CLI

Located in `config/gh/`

| File | Description |
|------|-------------|
| config.yml | GitHub CLI preferences |

**Note:** `hosts.yml` excluded (contains auth tokens)

### Karabiner Elements

Located in `config/karabiner/`

**Files:**
- `karabiner.json` - Main keyboard configuration
- `assets/complex_modifications/` - Custom key mappings
- `automatic_backups/` - Timestamped backups

### fd

Located in `config/fd/`

| File | Description |
|------|-------------|
| ignore | Global ignore patterns for fd |

## Utility Scripts

### User Scripts

Located in `bin/`

| Script | Description | Symlinked To |
|--------|-------------|--------------|
| fresh | Tmux workspace launcher | ~/bin/fresh |

### Setup Scripts

Located in `scripts/`

| Script | Description |
|--------|-------------|
| setup_symlinks.sh | Create/update all symlinks |
| tmux-fresh/ | Tmux AI workspace system |

### Tmux Fresh

Located in `scripts/tmux-fresh/`

| File | Description |
|------|-------------|
| start_tmux_homebase.sh | Main launcher script |
| agent_profiles.json | AI agent metadata |
| README.md | Usage documentation |

**Creates windows for:**
- Claude Code CLI
- Codex CLI
- Cliffy
- Lazygit
- Taskbook

## System Preferences

### macOS Defaults

Located at root: `macos-defaults.sh`

**Configures:**
- Dock behavior and position
- Finder preferences
- Keyboard settings
- Trackpad options
- Screenshot location
- Menu bar items

**Usage:**
```bash
./macos-defaults.sh
```

## Non-XDG Configs

Located in `app-configs/`

| File | Description |
|------|-------------|
| syncthing-config.xml | Syncthing device/folder config |
| obsidian-vaults.md | Vault locations reference |
| raycast-extensions.txt | Extension list |

## Documentation

Located in `docs/`

| File | Description |
|------|-------------|
| ENHANCEMENTS.md | Planned improvements |
| RETRO-OS.md | Terminal workspace documentation |
| tmux/*.md | Tmux workflow guides |

## Package Lists

| File | Description |
|------|-------------|
| Brewfile | Homebrew packages |
| Masfile | Mac App Store apps |
| npm-globals.txt | npm packages |
| cargo-installs.txt | Rust binaries |
| uv-tools.txt | Python tools |
| vscode-extensions.txt | VS Code extensions |

## Symlink Management

All symlinks created via:

```bash
./scripts/setup_symlinks.sh --force
```

**Strategy:**
- Shell configs → `~/`
- XDG configs → `~/.config/`
- Tmuxifier → `~/.tmuxifier`
- Git configs → `~/`
- SSH config → `~/.ssh/`
- User scripts → `~/bin/`
- VS Code → `~/Library/Application Support/Code/User/`

**Backups:**
- Existing files backed up to `~/dotfiles_backup_YYYYMMDD_HHMMSS/`
- Original files never overwritten without backup

## Security

Files excluded from version control:

- SSH private keys
- API tokens
- `.env` files
- Bitwarden sessions
- Claude Code directory
- Application auth tokens

See `.gitignore` for complete list.

## See Also

- [Homebrew Packages](Homebrew-Packages)
- [Development Tools](Development-Tools)
- [VS Code Extensions](VS-Code-Extensions)
- [Main README](https://github.com/bwl/dots#readme)
