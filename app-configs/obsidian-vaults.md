# Obsidian Vaults

My Obsidian vaults and their locations (for reference).

## Active Vaults

Based on `~/Library/Application Support/obsidian/obsidian.json`:

1. **getFresh**
   - Path: `/Users/bwl/Sync/getFresh`
   - Purpose: [Add description]

2. **Novels** (iCloud)
   - Path: `/Users/bwl/Library/Mobile Documents/iCloud~md~obsidian/Documents/Novels`
   - Purpose: Novel writing

3. **shared** (Developer)
   - Path: `/Users/bwl/Developer/novels/shared`
   - Purpose: Shared novel resources

4. **docs** (ratband) - Currently Open
   - Path: `/Users/bwl/Developer/ratband/docs`
   - Purpose: Project documentation

## Vault-Specific Settings

Each vault has its own `.obsidian/` folder with:
- `app.json` - Obsidian app settings
- `workspace.json` - Open files and layout
- `plugins/` - Installed community plugins
- `themes/` - Custom themes
- `hotkeys.json` - Custom hotkeys

## Setup on New Machine

1. Install Obsidian via Homebrew Cask (if not already in Brewfile)
2. Manually open each vault location above
3. Vault-specific settings will sync if vaults are synced via Syncthing/iCloud
4. Alternatively, copy `.obsidian/` folders if not synced

## Note

Obsidian settings are vault-specific and often synced with the vault content itself (via Syncthing, iCloud, etc.). This file is just for reference - not meant for automated restore.
