#!/bin/bash
# setup_symlinks.sh - Create symlinks from dotfiles to home directory
# Usage: ./setup_symlinks.sh [--force]
#
# This script creates symlinks from ~/dotfiles/ to their proper locations.
# Use --force to overwrite existing files (backs them up first).

set -e  # Exit on error

DOTFILES_DIR="$HOME/dotfiles"
BACKUP_DIR="$HOME/dotfiles_backup_$(date +%Y%m%d_%H%M%S)"
FORCE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
if [[ "$1" == "--force" ]]; then
    FORCE=true
    echo -e "${YELLOW}Force mode enabled - existing files will be backed up and replaced${NC}"
fi

# Function to create symlink with backup
create_symlink() {
    local source="$1"
    local target="$2"

    # Check if source exists
    if [[ ! -e "$source" ]]; then
        echo -e "${RED}✗ Source not found: $source${NC}"
        return 1
    fi

    # If target already exists
    if [[ -e "$target" ]] || [[ -L "$target" ]]; then
        if [[ "$FORCE" == true ]]; then
            # Create backup directory if needed
            mkdir -p "$BACKUP_DIR"

            # Backup existing file/link
            local backup_path="$BACKUP_DIR/$(basename "$target")"
            echo -e "${YELLOW}  Backing up existing: $target → $backup_path${NC}"
            mv "$target" "$backup_path"
        else
            echo -e "${YELLOW}⊙ Already exists (use --force to replace): $target${NC}"
            return 0
        fi
    fi

    # Create symlink
    ln -sf "$source" "$target"
    echo -e "${GREEN}✓ Linked: $target → $source${NC}"
}

echo "=================================================="
echo "  Dotfiles Symlink Setup"
echo "=================================================="
echo ""

# ============================================================================
# Shell Configurations
# ============================================================================
echo -e "${GREEN}Setting up shell configurations...${NC}"
create_symlink "$DOTFILES_DIR/shell/.zshrc" "$HOME/.zshrc"
create_symlink "$DOTFILES_DIR/shell/.zprofile" "$HOME/.zprofile"
create_symlink "$DOTFILES_DIR/shell/.zshenv" "$HOME/.zshenv"
create_symlink "$DOTFILES_DIR/shell/.p10k.zsh" "$HOME/.p10k.zsh"
echo ""

# ============================================================================
# Git Configurations
# ============================================================================
echo -e "${GREEN}Setting up git configurations...${NC}"
create_symlink "$DOTFILES_DIR/git/.gitconfig" "$HOME/.gitconfig"
create_symlink "$DOTFILES_DIR/git/.gitignore_global" "$HOME/.gitignore_global"
echo ""

# ============================================================================
# SSH Configuration (NOT keys!)
# ============================================================================
if [[ -f "$DOTFILES_DIR/ssh/config" ]]; then
    echo -e "${GREEN}Setting up SSH config...${NC}"
    mkdir -p "$HOME/.ssh"
    chmod 700 "$HOME/.ssh"
    create_symlink "$DOTFILES_DIR/ssh/config" "$HOME/.ssh/config"
    echo ""
fi

# ============================================================================
# Config Directory (individual items)
# ============================================================================
echo -e "${GREEN}Setting up ~/.config directories...${NC}"
mkdir -p "$HOME/.config"

# Symlink each config directory individually (except tmuxifier - handled separately)
for config_dir in "$DOTFILES_DIR/config"/*; do
    if [[ -d "$config_dir" ]]; then
        config_name=$(basename "$config_dir")
        # Skip tmuxifier - it goes to ~/.tmuxifier not ~/.config/
        if [[ "$config_name" != "tmuxifier" ]]; then
            create_symlink "$config_dir" "$HOME/.config/$config_name"
        fi
    fi
done
echo ""

# ============================================================================
# User Scripts (bin directory)
# ============================================================================
if [[ -d "$DOTFILES_DIR/bin" ]]; then
    echo -e "${GREEN}Setting up ~/bin directory...${NC}"
    create_symlink "$DOTFILES_DIR/bin" "$HOME/bin"
    echo ""
fi

# ============================================================================
# Tmuxifier (special case - goes to ~/.tmuxifier not ~/.config/)
# ============================================================================
if [[ -d "$DOTFILES_DIR/config/tmuxifier" ]]; then
    echo -e "${GREEN}Setting up tmuxifier...${NC}"
    create_symlink "$DOTFILES_DIR/config/tmuxifier" "$HOME/.tmuxifier"
    echo ""
fi

# ============================================================================
# Tmux compatibility symlink (for plugins that reference ~/.tmux/plugins/)
# ============================================================================
if [[ -d "$HOME/.config/tmux" ]] && [[ ! -e "$HOME/.tmux" ]]; then
    echo -e "${GREEN}Setting up tmux compatibility symlink...${NC}"
    ln -s "$HOME/.config/tmux" "$HOME/.tmux"
    echo -e "${GREEN}✓ Linked: ~/.tmux → ~/.config/tmux${NC}"
    echo ""
fi

# ============================================================================
# VS Code Settings
# ============================================================================
if [[ -f "$DOTFILES_DIR/vscode/settings.json" ]]; then
    echo -e "${GREEN}Setting up VS Code settings...${NC}"
    VSCODE_USER_DIR="$HOME/Library/Application Support/Code/User"
    if [[ -d "$VSCODE_USER_DIR" ]]; then
        create_symlink "$DOTFILES_DIR/vscode/settings.json" "$VSCODE_USER_DIR/settings.json"
    else
        echo -e "${YELLOW}⊙ VS Code user directory not found, skipping${NC}"
    fi
    echo ""
fi

# ============================================================================
# Summary
# ============================================================================
echo "=================================================="
echo -e "${GREEN}Symlink setup complete!${NC}"
echo "=================================================="
echo ""

if [[ -d "$BACKUP_DIR" ]]; then
    echo -e "${YELLOW}Backups saved to: $BACKUP_DIR${NC}"
    echo ""
fi

echo "Next steps:"
echo "  1. Review any backed up files if using --force"
echo "  2. Reload your shell: exec zsh"
echo "  3. Verify configs are working correctly"
echo ""
echo "To restore from backup:"
echo "  mv $BACKUP_DIR/* ~/"
echo ""
