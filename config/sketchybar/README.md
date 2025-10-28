# SketchyBar Configuration

Minimal starter configuration for SketchyBar - a highly customizable macOS status bar.

## Structure

- `sketchybarrc` - Main configuration file (bash script)
- `colors.sh` - Color scheme definitions (dark Catppuccin-inspired)
- `icons.sh` - Nerd Font icon glyphs
- `plugins/` - Custom plugin scripts

## Included Plugins

### Left Side
- **space.sh** - Workspace/desktop indicators (1-10)
- **front_app.sh** - Currently focused application

### Right Side
- **clock.sh** - Time and date (`Mon Oct 25  14:30`)
- **battery.sh** - Battery percentage with charging status
- **network.sh** - WiFi SSID or connection status
- **cpu.sh** - CPU utilization percentage
- **memory.sh** - RAM usage percentage

### Auto-Hide System
- **auto_hide.sh** - Intelligent auto-hide with cursor bouncer (see below)

## Cursor Bouncer & Auto-Hide

Custom solution to prevent accidental macOS menu bar triggers while keeping SketchyBar always accessible.

### The Problem

When hiding the native macOS menu bar (as recommended in [SketchyBar docs](https://felixkratz.github.io/SketchyBar/setup#hiding-the-original-macos-bar)), it's too easy to accidentally trigger the macOS menu when trying to interact with SketchyBar items at the top of the screen. This creates a frustrating UX where you're constantly fighting between two menu bars.

### The Solution

This config includes a **cursor bouncer** that prevents accidental menu bar access:

**Features:**
- **Cursor Bouncer**: Prevents cursor from entering top 5px zone (blocks accidental macOS menu bar)
- **Command Key Override**: Hold Command to bypass bouncer and access macOS menu bar
- **Menu Detection**: Keeps SketchyBar hidden while macOS menus are open (no overlap)
- **Smart Hide**: Hides items but keeps bar background visible (prevents wallpaper flash)
- **Dual-Threshold Hysteresis**: Bar hides at 3px from top, shows at 44px (prevents flickering)

**Result**: Natural, polished UX where SketchyBar stays visible and accessible, but you can still easily access the macOS menu bar when needed (just hold Command and move to top).

### Architecture

**Components:**
- `helpers/cursor_monitor.app/` - Signed macOS app bundle (Swift)
  - Monitors cursor position using AppKit/CoreGraphics
  - Bounces cursor at top edge using `CGWarpMouseCursorPosition`
  - Detects Command key press via `NSEvent.modifierFlags`
  - Detects open menus via `CGWindowListCopyWindowInfo`
  - Sends custom SketchyBar events: `cursor_at_top` / `cursor_away_from_top`

- `plugins/auto_hide.sh` - Event handler
  - Listens for cursor position events
  - Hides/shows all items via `sketchybar --set '/.*/' drawing=off/on`
  - Keeps bar background visible (no wallpaper flash)

- `sketchybarrc` - Integration
  - Registers custom events
  - Launches cursor monitor on startup
  - Manages monitor process lifecycle

**Thresholds** (tuned for MacBook Air 1710x1112):
- `topEdgeThreshold = 3px` - Auto-hide trigger
- `bottomEdgeActiveThreshold = 44px` - Auto-show trigger
- `bouncerThreshold = 5px` - Cursor bounce distance
- `bounceTargetOffset = 6px` - Bounce destination

### Permissions

The cursor bouncer requires **macOS Accessibility permissions** to move the cursor:

1. Run `sketchybar --reload`
2. Grant permission in: **System Settings > Privacy & Security > Accessibility**
3. Enable **"Cursor Monitor"**
4. Reload again

If permissions are denied, the bouncer gracefully disables (auto-hide still works, just no cursor bouncing).

### Technical Details

**Coordinate System Fix:**
- `NSEvent.mouseLocation` uses Cocoa coordinates (Y=0 at bottom, increases upward)
- `CGWarpMouseCursorPosition` uses CG coordinates (Y=0 at top, increases downward)
- Conversion: `CG_Y = screenHeight - Cocoa_Y`

**Menu Detection:**
- Scans window list for windows with level ≥ 101 (`kCGPopUpMenuWindowLevel`)
- Keeps SketchyBar hidden until menu closes (prevents overlap)

**App Bundle:**
- Compiled Swift binary with ad-hoc code signature
- Appears as "Cursor Monitor" in System Settings
- Auto-starts with SketchyBar, auto-restarts on reload

### Commits

Implementation across 4 commits:
- `951346d` - Initial auto-hide with hysteresis
- `55328e4` - Added menu detection
- `8b43b0f` - Added cursor bouncer with Command override + app bundle
- `da4fd57` - Fixed wallpaper flash (keep background visible)

## Usage

```bash
# Reload after editing configs
sketchybar --reload

# Manage service
brew services start sketchybar
brew services stop sketchybar
brew services restart sketchybar

# Debug mode (foreground)
sketchybar --config ~/.config/sketchybar/sketchybarrc
```

## Customization

### Colors
Edit `colors.sh` to customize your theme. Current palette uses:
- Dark backgrounds (`#1e1e2e`, `#313244`)
- Light text (`#cdd6f4`, `#bac2de`)
- Blue accent (`#89b4fa`)
- Status colors (red/yellow/green for warnings)

### Icons
Edit `icons.sh` to change icons. Requires Hack Nerd Font (installed via Brewfile).
Icon reference: https://www.nerdfonts.com/cheat-sheet

### Adding Custom Plugins

1. Create executable script in `plugins/`:
   ```bash
   touch plugins/my_plugin.sh
   chmod +x plugins/my_plugin.sh
   ```

2. Write plugin logic:
   ```bash
   #!/bin/bash
   # Available environment variables:
   # $NAME - item name
   # $INFO - context info (varies by event)
   # $SENDER - event sender
   # $SELECTED - selected state (for spaces)

   sketchybar --set $NAME label="My Value"
   ```

3. Add to `sketchybarrc`:
   ```bash
   sketchybar --add item my_item right \
       --set my_item \
           update_freq=10 \
           icon="󰊠" \
           script="$PLUGIN_DIR/my_plugin.sh"
   ```

4. Reload: `sketchybar --reload`

## Resources

- Official docs: https://felixkratz.github.io/SketchyBar/
- Example configs: https://github.com/FelixKratz/SketchyBar/discussions/47
- Plugin examples: https://github.com/FelixKratz/SketchyBar/discussions/12
