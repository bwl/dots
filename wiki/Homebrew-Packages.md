# Homebrew Packages

55 formulae installed via Homebrew.

## Development Languages & Runtimes

| Package | Description |
|---------|-------------|
| rust | Systems programming language |
| rust-analyzer | Rust LSP server |
| oven-sh/bun/bun | Fast JavaScript runtime & package manager |
| uv | Fast Python package installer |
| pyenv-virtualenv | Python version & environment management |

## Text Editors

| Package | Description |
|---------|-------------|
| neovim | Hyperextensible Vim-based text editor |
| msedit | Retro Microsoft terminal editor |

## CLI Utilities

| Package | Description |
|---------|-------------|
| bat | Cat clone with syntax highlighting |
| fd | Fast alternative to find |
| fzf | Fuzzy finder |
| ripgrep | Fast grep alternative (rg) |
| eza | Modern replacement for ls |
| jq | JSON processor |
| sd | Sed alternative |
| tree | Directory tree visualization |
| wget | Network downloader |
| yt-dlp | Video downloader |

## File Managers

| Package | Description |
|---------|-------------|
| yazi | Modern terminal file manager |

## AI/LLM Tools

| Package | Description |
|---------|-------------|
| qwen-code | AI coding assistant |
| opencode | AI tooling evaluation |
| charmbracelet/tap/crush | Charm log viewer |
| specify | GitHub AI coding tool |

## Git Tools

| Package | Description |
|---------|-------------|
| gh | GitHub CLI |
| lazygit | Terminal UI for git |
| tig | Text-mode interface for git |
| git-lfs | Large File Storage for Git |
| git-filter-repo | Quickly rewrite git repository history |

## System Monitoring & Analysis

| Package | Description |
|---------|-------------|
| htop | Interactive process viewer |
| mactop | macOS activity monitor in terminal |
| ncdu | Disk usage analyzer with ncurses interface |
| procs | Modern replacement for ps |

## Build Tools & Task Runners

| Package | Description |
|---------|-------------|
| go-task | Task runner / Make alternative |
| just | Command runner (Makefile alternative) |

## Code Quality & Linting

| Package | Description |
|---------|-------------|
| shellcheck | Shell script static analysis |
| golangci-lint | Fast Go linters runner |
| taplo | TOML toolkit |
| tree-sitter-cli | Parser generator & query tool |
| ast-grep | Structural code search & manipulation |

## Terminal & Shell

| Package | Description |
|---------|-------------|
| tmux | Terminal multiplexer |
| editorconfig | Maintain consistent coding styles |
| gum | Glamorous shell scripts (interactive prompts) |
| sampler | Terminal dashboard visualization |
| zoxide | Smarter cd command (autojump) |

## Productivity & Information

| Package | Description |
|---------|-------------|
| glow | Markdown renderer for CLI |
| television | Fuzzy finder for everything |
| jbreckmckye/formulae/daylight | Sun tracker for dashboard |
| fastfetch | System information tool |
| fortune | Random quotes for dashboard |

## Package Management Tools

| Package | Description |
|---------|-------------|
| mas | Mac App Store CLI |
| bitwarden-cli | Password manager CLI |
| cosmincatalin/tap/brew-explorer | TUI for Homebrew |

## Games & Entertainment

| Package | Description |
|---------|-------------|
| angband | Roguelike dungeon exploration game |

## Installation

```bash
cd ~/dotfiles
brew bundle install --file=Brewfile
```

## Update Package List

```bash
brew bundle dump --force --file=Brewfile
```
