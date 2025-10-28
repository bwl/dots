#!/bin/bash

# SketchyBar auto-hide plugin
# Responds to cursor position events from cursor_monitor
# Hides items when cursor is near top edge (to reveal macOS menu bar)
# Keeps bar background visible to prevent wallpaper flash

case "$SENDER" in
  "cursor_at_top")
    # Cursor is near top edge - hide all items but keep bar background visible
    # This prevents wallpaper flash when macOS menu bar appears
    sketchybar --set '/.*/' drawing=off
    ;;
  "cursor_away_from_top")
    # Cursor moved away from top edge - show all items
    sketchybar --set '/.*/' drawing=on
    ;;
esac
