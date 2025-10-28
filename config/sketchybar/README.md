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
