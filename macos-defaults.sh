#!/bin/bash
# macos-defaults.sh - Configure macOS system preferences
# Run this script after a fresh macOS install to restore your preferred settings
#
# Usage: ./macos-defaults.sh
#
# Note: Some changes require logout/restart to take effect

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=================================================="
echo -e "${BLUE}macOS System Preferences Configuration${NC}"
echo "=================================================="
echo ""

echo -e "${YELLOW}⚠️  Some changes require logout/restart to take effect${NC}"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted"
    exit 0
fi

# ============================================================================
# Dock
# ============================================================================
echo -e "${GREEN}Configuring Dock...${NC}"

# Auto-hide the Dock
defaults write com.apple.dock autohide -bool true

# Set Dock size
# defaults write com.apple.dock tilesize -int 48

# Disable magnification
defaults write com.apple.dock magnification -bool false

# Position (left, bottom, right)
defaults write com.apple.dock orientation -string "bottom"

# Minimize effect (genie, scale, suck)
defaults write com.apple.dock mineffect -string "genie"

# Don't show recent apps
defaults write com.apple.dock show-recents -bool false

# Speed up Mission Control animations
defaults write com.apple.dock expose-animation-duration -float 0.1

echo "  ✓ Dock configured"

# ============================================================================
# Finder
# ============================================================================
echo -e "${GREEN}Configuring Finder...${NC}"

# Show hidden files
defaults write com.apple.finder AppleShowAllFiles -bool true

# Show file extensions
defaults write NSGlobalDomain AppleShowAllExtensions -bool true

# Show path bar
defaults write com.apple.finder ShowPathbar -bool true

# Show status bar
defaults write com.apple.finder ShowStatusBar -bool true

# Default view style (icnv, clmv, Flwv, Nlsv)
# icnv = Icon, clmv = Column, Flwv = Cover Flow, Nlsv = List
defaults write com.apple.finder FXPreferredViewStyle -string "clmv"

# Search current folder by default
defaults write com.apple.finder FXDefaultSearchScope -string "SCcf"

# Disable warning when changing file extension
defaults write com.apple.finder FXEnableExtensionChangeWarning -bool false

# Keep folders on top
defaults write com.apple.finder _FXSortFoldersFirst -bool true

echo "  ✓ Finder configured"

# ============================================================================
# Screenshots
# ============================================================================
echo -e "${GREEN}Configuring Screenshots...${NC}"

# Save screenshots to Downloads folder
defaults write com.apple.screencapture location -string "${HOME}/Downloads"

# Save screenshots as PNG (other options: BMP, GIF, JPG, PDF, TIFF)
defaults write com.apple.screencapture type -string "png"

# Disable shadow in screenshots
defaults write com.apple.screencapture disable-shadow -bool true

echo "  ✓ Screenshots configured"

# ============================================================================
# Keyboard & Input
# ============================================================================
echo -e "${GREEN}Configuring Keyboard...${NC}"

# Fast key repeat rate (1 = fastest, 2 = fast)
defaults write NSGlobalDomain KeyRepeat -int 2

# Short delay until key repeat (15 = shortest)
defaults write NSGlobalDomain InitialKeyRepeat -int 15

# Disable automatic capitalization
defaults write NSGlobalDomain NSAutomaticCapitalizationEnabled -bool false

# Disable smart dashes
defaults write NSGlobalDomain NSAutomaticDashSubstitutionEnabled -bool false

# Disable automatic period substitution
defaults write NSGlobalDomain NSAutomaticPeriodSubstitutionEnabled -bool false

# Disable smart quotes
defaults write NSGlobalDomain NSAutomaticQuoteSubstitutionEnabled -bool false

# Disable auto-correct
defaults write NSGlobalDomain NSAutomaticSpellingCorrectionEnabled -bool false

echo "  ✓ Keyboard configured"

# ============================================================================
# Trackpad
# ============================================================================
echo -e "${GREEN}Configuring Trackpad...${NC}"

# Enable tap to click
defaults write com.apple.driver.AppleBluetoothMultitouch.trackpad Clicking -bool true
defaults write NSGlobalDomain com.apple.mouse.tapBehavior -int 1
defaults -currentHost write NSGlobalDomain com.apple.mouse.tapBehavior -int 1

# Natural scrolling (true = macOS default, false = traditional)
# defaults write NSGlobalDomain com.apple.swipescrolldirection -bool false

echo "  ✓ Trackpad configured"

# ============================================================================
# Menu Bar
# ============================================================================
echo -e "${GREEN}Configuring Menu Bar...${NC}"

# Auto-hide the menu bar (for use with SketchyBar)
defaults write NSGlobalDomain _HIHideMenuBar -bool true

# Show battery percentage
defaults write com.apple.menuextra.battery ShowPercent -string "YES"

# Show date and time
defaults write com.apple.menuextra.clock DateFormat -string "EEE MMM d  h:mm a"

echo "  ✓ Menu Bar configured (auto-hide enabled for SketchyBar)"

# ============================================================================
# General System
# ============================================================================
echo -e "${GREEN}Configuring System...${NC}"

# Disable the sound effects on boot
# sudo nvram SystemAudioVolume=" "

# Reduce Motion (disables space switching slide animation, uses fade instead)
defaults write com.apple.universalaccess reduceMotion -bool true

# Disable window animations
defaults write NSGlobalDomain NSAutomaticWindowAnimationsEnabled -bool false

# Expand save panel by default
defaults write NSGlobalDomain NSNavPanelExpandedStateForSaveMode -bool true
defaults write NSGlobalDomain NSNavPanelExpandedStateForSaveMode2 -bool true

# Expand print panel by default
defaults write NSGlobalDomain PMPrintingExpandedStateForPrint -bool true
defaults write NSGlobalDomain PMPrintingExpandedStateForPrint2 -bool true

# Save to disk (not to iCloud) by default
defaults write NSGlobalDomain NSDocumentSaveNewDocumentsToCloud -bool false

# Disable Resume system-wide
defaults write com.apple.systempreferences NSQuitAlwaysKeepsWindows -bool false

# Disable automatic termination of inactive apps
defaults write NSGlobalDomain NSDisableAutomaticTermination -bool true

echo "  ✓ System configured"

# ============================================================================
# Activity Monitor
# ============================================================================
echo -e "${GREEN}Configuring Activity Monitor...${NC}"

# Show all processes
defaults write com.apple.ActivityMonitor ShowCategory -int 0

# Sort by CPU usage
defaults write com.apple.ActivityMonitor SortColumn -string "CPUUsage"
defaults write com.apple.ActivityMonitor SortDirection -int 0

echo "  ✓ Activity Monitor configured"

# ============================================================================
# Terminal
# ============================================================================
echo -e "${GREEN}Configuring Terminal...${NC}"

# Only use UTF-8
defaults write com.apple.terminal StringEncodings -array 4

# Enable Secure Keyboard Entry
defaults write com.apple.terminal SecureKeyboardEntry -bool true

echo "  ✓ Terminal configured"

# ============================================================================
# Restart Affected Applications
# ============================================================================
echo ""
echo -e "${YELLOW}Restarting affected applications...${NC}"

for app in "Dock" "Finder" "SystemUIServer"; do
    killall "${app}" &> /dev/null || true
done

echo ""
echo "=================================================="
echo -e "${GREEN}✓ macOS preferences configured!${NC}"
echo "=================================================="
echo ""
echo -e "${YELLOW}Note: Some changes require a logout/restart to take full effect${NC}"
echo ""
echo "Changes applied:"
echo "  • Dock: Auto-hide enabled, no magnification, faster animations"
echo "  • Finder: Show hidden files, extensions, path bar"
echo "  • Screenshots: Save to Downloads as PNG, no shadow"
echo "  • Keyboard: Fast key repeat, disabled auto-correct"
echo "  • Trackpad: Tap to click enabled"
echo "  • Menu Bar: Auto-hide enabled (for SketchyBar), show battery %, date/time"
echo "  • System: Reduced motion, disabled window animations, expanded panels"
echo ""
