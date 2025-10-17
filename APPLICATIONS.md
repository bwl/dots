# Application Preferences Guide

Complete guide for managing application configurations beyond standard dotfiles.

---

## Table of Contents

- [Overview](#overview)
- [What's Tracked](#whats-tracked)
- [Security Considerations](#security-considerations)
- [Per-Application Guide](#per-application-guide)
- [macOS System Preferences](#macos-system-preferences)
- [Finding App Configs](#finding-app-configs)

---

## Overview

macOS applications store preferences in several locations:
- `~/Library/Preferences/` - Binary .plist files
- `~/Library/Application Support/` - App-specific data/configs
- `~/.config/` - XDG-compliant configs (increasingly common)
- Hardcoded in app bundles (can't be backed up)

This repo tracks **reproducible** configs while excluding secrets and binaries.

---

## What's Tracked

| Item | Location | Size | Notes |
|------|----------|------|-------|
| VS Code extensions | `vscode-extensions.txt` | 108 extensions | List only |
| VS Code settings | `vscode/settings.json` | Small | Full config |
| Syncthing config | `app-configs/syncthing-config.xml` | 12KB | Has device IDs |
| Raycast extensions | `app-configs/raycast-extensions.txt` | 41 IDs | List only |
| Obsidian vaults | `app-configs/obsidian-vaults.md` | Doc | Reference only |
| macOS defaults | `macos-defaults.sh` | Script | System prefs |

---

## Security Considerations

### ⚠️ DO NOT Track These

**Raycast config.json** - Contains access token
```bash
# Location: ~/.config/raycast/config.json
# Contains: "accesstoken": "rca_..."
# Solution: Already in .gitignore
```

**Syncthing Device IDs** - Semi-sensitive
```bash
# In: app-configs/syncthing-config.xml
# Contains: <device id="...">
# These identify your devices - not critical but prefer private repo
```

**Bitwarden Session** - Critical secret
```bash
# Variable: $BW_SESSION
# Never commit to dotfiles
# See BITWARDEN_GUIDE.md
```

### ✅ Safe to Track

- Extension/plugin lists (no auth tokens)
- Editor settings (no API keys)
- System preferences (public settings)
- App configurations (after removing secrets)

---

## Per-Application Guide

### VS Code

**What's tracked:**
- `vscode/settings.json` - Editor preferences
- `vscode-extensions.txt` - 108 installed extensions

**Restore:**
```bash
# Settings (via symlink - automatic)
ln -sf ~/dotfiles/vscode/settings.json \
  "$HOME/Library/Application Support/Code/User/settings.json"

# Extensions
cat ~/dotfiles/vscode-extensions.txt | xargs -L 1 code --install-extension
```

**Update:**
```bash
code --list-extensions > ~/dotfiles/vscode-extensions.txt
```

---

### Raycast

**What's tracked:**
- `app-configs/raycast-extensions.txt` - 41 extension UUIDs

**What's NOT tracked:**
- config.json (contains access token!)
- Extension binaries (341MB)

**Location:**
- Config: `~/.config/raycast/`
- Extensions: `~/.config/raycast/extensions/`

**Manual setup:**
1. Install Raycast via Homebrew Cask
2. Sign in with Raycast account
3. Extensions will sync automatically via Raycast account

**Note:** Extension list is for reference. Raycast syncs extensions via cloud account.

---

### Syncthing

**What's tracked:**
- `app-configs/syncthing-config.xml` - Full configuration

**Location:**
- `~/Library/Application Support/Syncthing/config.xml`

**Restore:**
```bash
# Stop Syncthing first
# Copy config
cp ~/dotfiles/app-configs/syncthing-config.xml \
  "$HOME/Library/Application Support/Syncthing/config.xml"

# Restart Syncthing
```

**Security note:**
- Config contains device IDs (semi-sensitive)
- Keep dotfiles repo private, or sanitize device IDs before committing

**Update:**
```bash
cp "$HOME/Library/Application Support/Syncthing/config.xml" \
  ~/dotfiles/app-configs/syncthing-config.xml
```

---

### Obsidian

**What's tracked:**
- `app-configs/obsidian-vaults.md` - Reference doc listing vault locations

**What's NOT tracked:**
- Vault contents (managed separately - Syncthing/iCloud/Git)
- Per-vault `.obsidian/` settings (synced with vault)

**Location:**
- App config: `~/Library/Application Support/obsidian/obsidian.json`
- Vault-specific settings: `<vault-path>/.obsidian/`

**Manual setup:**
1. Install Obsidian (already in Homebrew Cask if added to Brewfile)
2. Open each vault manually (paths in obsidian-vaults.md)
3. Vault settings restore automatically if vault is synced

**Philosophy:**
Obsidian settings are vault-specific and usually synced with vault content. This dotfile tracks vault locations for reference, not automated restore.

---

### Ghostty

**What's tracked:**
- `config/ghostty/` - Full terminal config

**Location:**
- `~/.config/ghostty/`

**Restore:**
```bash
# Via symlink (automatic with setup_symlinks.sh)
ln -sf ~/dotfiles/config/ghostty ~/.config/ghostty
```

---

### Neovim

**What's tracked:**
- `config/nvim/` - Full Neovim configuration

**Location:**
- `~/.config/nvim/`

**Restore:**
```bash
# Via symlink (automatic with setup_symlinks.sh)
ln -sf ~/dotfiles/config/nvim ~/.config/nvim
```

---

### Karabiner-Elements

**What's tracked:**
- `config/karabiner/` - Keyboard customization

**Location:**
- `~/.config/karabiner/`

**Restore:**
```bash
# Via symlink (automatic with setup_symlinks.sh)
ln -sf ~/dotfiles/config/karabiner ~/.config/karabiner
```

---

### tmux

**What's tracked:**
- `config/tmux/` - Terminal multiplexer config

**Location:**
- `~/.config/tmux/`

**Restore:**
```bash
# Via symlink (automatic with setup_symlinks.sh)
ln -sf ~/dotfiles/config/tmux ~/.config/tmux
```

---

## macOS System Preferences

**What's tracked:**
- `macos-defaults.sh` - Executable script with `defaults write` commands

**Run on fresh install:**
```bash
cd ~/dotfiles
./macos-defaults.sh
```

**Settings configured:**
- **Dock:** Auto-hide, position, size, no magnification
- **Finder:** Show hidden files, extensions, path bar, status bar
- **Screenshots:** Save to ~/Downloads as PNG, no shadow
- **Keyboard:** Fast key repeat, disabled auto-correct/smart features
- **Trackpad:** Tap to click enabled
- **Menu Bar:** Show battery %, date/time format
- **System:** Expanded save/print panels, save to disk (not iCloud)

**Capture current settings:**
```bash
# Example: Read current Dock settings
defaults read com.apple.dock

# Example: Export specific setting
defaults read com.apple.dock autohide
```

**Some settings require logout/restart to take effect.**

---

## Finding App Configs

### Common Locations

1. **~/.config/** - Modern apps (XDG Base Directory spec)
   ```bash
   ls ~/.config/
   ```

2. **~/Library/Application Support/** - macOS apps
   ```bash
   ls "$HOME/Library/Application Support/"
   ```

3. **~/Library/Preferences/** - Binary .plist files
   ```bash
   ls ~/Library/Preferences/ | grep com.yourapp
   ```

4. **~/.appname** - Old-school dotfiles
   ```bash
   ls -la ~ | grep "^\."
   ```

### Reading .plist Files

```bash
# Read entire plist
defaults read com.apple.dock

# Read specific key
defaults read com.apple.dock autohide

# Export as XML
plutil -convert xml1 ~/Library/Preferences/com.apple.dock.plist -o -
```

### Searching for App Configs

```bash
# Find app-specific files
find ~/Library -iname "*appname*" 2>/dev/null

# Search preferences
ls ~/Library/Preferences/ | grep -i "appname"

# Check .config
ls ~/.config/ | grep -i "appname"
```

---

## Adding New App Configs

### Checklist

1. **Locate the config:**
   ```bash
   # Check common locations
   ls ~/.config/appname
   ls "$HOME/Library/Application Support/appname"
   defaults read com.company.appname
   ```

2. **Check for secrets:**
   ```bash
   # Search for tokens, keys, passwords
   grep -i "token\|key\|password\|secret" config-file
   ```

3. **Decide what to track:**
   - ✅ Settings, preferences, keybindings
   - ✅ Plugin/extension lists
   - ❌ Auth tokens, API keys
   - ❌ Large binaries/caches
   - ❌ Temporary files

4. **Copy to dotfiles:**
   ```bash
   # For configs
   cp -r ~/.config/appname ~/dotfiles/config/

   # For app-specific
   cp file ~/dotfiles/app-configs/appname-config.ext
   ```

5. **Add to .gitignore if needed:**
   ```bash
   # If config contains secrets that can't be easily removed
   echo "app-configs/appname-config.ext" >> ~/dotfiles/.gitignore
   ```

6. **Document in this file:**
   - Add section with location, restore steps, update steps
   - Note any security considerations

7. **Update README.md:**
   - Add to "What's Included" section if significant

---

## Apps NOT Tracked

These apps either:
- Store settings in iCloud (synced automatically)
- Use proprietary formats (can't easily version control)
- Contain too many secrets
- Are too large

**Examples:**
- **Xcode:** Settings sync via iCloud, too large
- **Logic Pro / GarageBand:** Audio plugins, samples (huge)
- **Raycast config.json:** Contains access token
- **Browser profiles:** Too large, contain secrets
- **Mail accounts:** Credentials (use Bitwarden + manual setup)
- **App Store apps:** Licenses managed by App Store

---

## Philosophy

**Track:**
- Reproducible configurations
- Settings you customize
- Lists of installed extensions/plugins

**Don't Track:**
- Secrets (API keys, tokens, passwords)
- Large binaries (extension .app files, plugins)
- Auto-generated files (caches, logs)
- Personal data (vault contents, documents)

**Rule of Thumb:**
If you can't safely push it to a public GitHub repo (even without secrets), reconsider tracking it.

---

**Last Updated:** 2025-10-16
