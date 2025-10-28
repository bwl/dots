# SketchyBar Installation Guide

Quick guide to installing and configuring SketchyBar on a new system.

## Prerequisites

Ensure you have "Displays have separate Spaces" enabled:
- System Settings → Desktop & Dock → Mission Control → "Displays have separate Spaces" (enabled)

## Installation Steps

### 1. Install via Homebrew

From your dotfiles directory:

```bash
cd ~/dotfiles
brew bundle install --file=Brewfile
```

This installs:
- SketchyBar (from FelixKratz/formulae tap)
- Hack Nerd Font (required for icons)

### 2. Create Symlinks

Run the symlink setup script (SketchyBar config already included):

```bash
./scripts/setup_symlinks.sh --force
```

This creates: `~/.config/sketchybar/` → `~/dotfiles/config/sketchybar/`

### 3. Hide Native Menu Bar (Optional but Recommended)

Run the macOS defaults script:

```bash
./macos-defaults.sh
```

Or manually:
- **Sonoma/Sequoia**: System Settings → Control Center → "Automatically hide and show the menu bar"
- **Ventura**: System Settings → Desktop & Dock → "Automatically hide and show the menu bar"
- **Pre-Ventura**: System Preferences → Dock & Menu Bar

### 4. Start SketchyBar

```bash
brew services start sketchybar
```

SketchyBar will now start automatically on login.

## Verification

Check if SketchyBar is running:

```bash
brew services list | grep sketchybar
```

Should show `started`.

## Troubleshooting

### Bar not visible
- Ensure menu bar auto-hide is enabled
- Try restarting: `brew services restart sketchybar`
- Check logs: `tail -f /tmp/sketchybar.log`

### Icons not showing
- Verify Hack Nerd Font is installed: `fc-list | grep Hack`
- Install manually if needed: `brew install --cask font-hack-nerd-font`

### Plugins not working
- Verify permissions: `ls -la ~/.config/sketchybar/plugins/`
- Make executable: `chmod +x ~/.config/sketchybar/plugins/*.sh`

### Reset to defaults
```bash
brew services stop sketchybar
rm -rf ~/.config/sketchybar
./scripts/setup_symlinks.sh --force
brew services start sketchybar
```

## Next Steps

- Customize colors in `~/.config/sketchybar/colors.sh`
- Add custom plugins in `~/.config/sketchybar/plugins/`
- Modify bar layout in `~/.config/sketchybar/sketchybarrc`
- Reload after changes: `sketchybar --reload`
