#!/bin/bash
# install.sh - Bootstrap script for fresh macOS systems
# This script sets up a complete development environment from scratch
#
# Usage: ./install.sh

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

DOTFILES_DIR="$HOME/dotfiles"

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
    echo ""
    echo "=================================================="
    echo -e "${BLUE}  $1${NC}"
    echo "=================================================="
    echo ""
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# ============================================================================
# Checks
# ============================================================================

print_header "Dotfiles Bootstrap - Fresh macOS Setup"

# Check if running on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    print_error "This script is designed for macOS only"
    exit 1
fi

# Check if Xcode Command Line Tools are installed
if ! xcode-select -p &> /dev/null; then
    print_warning "Xcode Command Line Tools not found"
    echo "Installing Xcode Command Line Tools..."
    xcode-select --install
    echo "Please complete the installation and run this script again"
    exit 0
fi
print_success "Xcode Command Line Tools installed"

# Check if we're in the dotfiles directory
if [[ ! -f "$DOTFILES_DIR/Brewfile" ]]; then
    print_error "Brewfile not found in $DOTFILES_DIR"
    print_error "Please clone your dotfiles repo to ~/dotfiles first"
    exit 1
fi

# ============================================================================
# Install Homebrew
# ============================================================================

print_header "Step 1: Installing Homebrew"

if command -v brew &> /dev/null; then
    print_success "Homebrew already installed"
    brew --version
else
    echo "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

    # Add Homebrew to PATH for Apple Silicon Macs
    if [[ $(uname -m) == "arm64" ]]; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "$HOME/.zprofile"
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi

    print_success "Homebrew installed"
fi

# ============================================================================
# Install Packages from Brewfile
# ============================================================================

print_header "Step 2: Installing Homebrew Packages"

echo "This will install 95 formulae from your Brewfile..."
echo "This may take 15-30 minutes depending on your connection."
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    cd "$DOTFILES_DIR"
    brew bundle install --file=Brewfile
    print_success "Homebrew packages installed"
else
    print_warning "Skipping Homebrew package installation"
fi

# ============================================================================
# Install Mac App Store Apps
# ============================================================================

print_header "Step 3: Installing Mac App Store Apps"

if command -v mas &> /dev/null; then
    # Check if signed into App Store
    if mas account &> /dev/null; then
        echo "Installing apps from Masfile..."
        echo ""

        while IFS= read -r line; do
            # Extract app ID (first field)
            app_id=$(echo "$line" | awk '{print $1}')
            app_name=$(echo "$line" | awk '{$1=""; print $0}' | xargs)

            if [[ -n "$app_id" ]] && [[ "$app_id" =~ ^[0-9]+$ ]]; then
                echo "Installing: $app_name"
                mas install "$app_id" || print_warning "Failed to install $app_name (may already be installed)"
            fi
        done < "$DOTFILES_DIR/Masfile"

        print_success "Mac App Store apps processed"
    else
        print_warning "Not signed into Mac App Store"
        print_warning "Please sign in via System Settings > App Store"
        print_warning "Then manually run: cat ~/dotfiles/Masfile | awk '{print \$1}' | xargs mas install"
    fi
else
    print_warning "mas not installed (should have been installed via Brewfile)"
fi

# ============================================================================
# Setup Bitwarden CLI
# ============================================================================

print_header "Step 4: Setting up Bitwarden CLI"

if command -v bw &> /dev/null; then
    print_success "Bitwarden CLI installed"
    echo ""
    echo "To complete Bitwarden setup:"
    echo "  1. Login: bw login"
    echo "  2. Unlock: export BW_SESSION=\$(bw unlock --raw)"
    echo "  3. Store your secrets in Bitwarden"
    echo "  4. Update shell configs to reference them"
    echo ""
    print_warning "Remember: NEVER commit secrets to git!"
else
    print_warning "Bitwarden CLI not installed"
fi

# ============================================================================
# Setup Shell (Oh-My-Zsh + Powerlevel10k)
# ============================================================================

print_header "Step 5: Setting up Zsh with Powerlevel10k"

# Install Oh-My-Zsh if not present
if [[ ! -d "$HOME/.oh-my-zsh" ]]; then
    echo "Installing Oh-My-Zsh..."
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended
    print_success "Oh-My-Zsh installed"
else
    print_success "Oh-My-Zsh already installed"
fi

# Install Powerlevel10k theme
P10K_DIR="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/themes/powerlevel10k"
if [[ ! -d "$P10K_DIR" ]]; then
    echo "Installing Powerlevel10k theme..."
    git clone --depth=1 https://github.com/romkatv/powerlevel10k.git "$P10K_DIR"
    print_success "Powerlevel10k installed"
else
    print_success "Powerlevel10k already installed"
fi

# ============================================================================
# Create Symlinks
# ============================================================================

print_header "Step 6: Creating Symlinks"

echo "This will create symlinks from ~/dotfiles to your home directory."
echo "Existing files will be backed up."
echo ""
read -p "Create symlinks? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    bash "$DOTFILES_DIR/scripts/setup_symlinks.sh" --force
    print_success "Symlinks created"
else
    print_warning "Skipping symlink creation"
    echo "To create symlinks later, run:"
    echo "  bash ~/dotfiles/scripts/setup_symlinks.sh --force"
fi

# ============================================================================
# Final Steps
# ============================================================================

print_header "Installation Complete!"

echo "Next steps:"
echo ""
echo "1. ${YELLOW}Setup Bitwarden:${NC}"
echo "   bw login"
echo "   export BW_SESSION=\$(bw unlock --raw)"
echo ""
echo "2. ${YELLOW}Move secrets from .zshenv to Bitwarden${NC}"
echo "   See ~/dotfiles/PLAN.md for detailed instructions"
echo ""
echo "3. ${YELLOW}Reload your shell:${NC}"
echo "   exec zsh"
echo ""
echo "4. ${YELLOW}Configure Powerlevel10k (if prompted):${NC}"
echo "   p10k configure"
echo ""
echo "5. ${YELLOW}Optional - Initialize git repo:${NC}"
echo "   cd ~/dotfiles"
echo "   git init"
echo "   git add ."
echo "   git commit -m \"Initial dotfiles commit\""
echo "   gh repo create dotfiles --private --source=. --push"
echo ""
echo "6. ${YELLOW}Review and customize:${NC}"
echo "   - Check configs in ~/.config/"
echo "   - Customize shell aliases in ~/.zshrc"
echo "   - Set up any remaining app-specific configs"
echo ""

print_success "All done! Enjoy your new setup! 🚀"
echo ""
