#!/bin/bash

# SketchyBar auto-hide plugin
# Responds to cursor position events from cursor_monitor
# Hides items when cursor is near top edge (to reveal macOS menu bar)
# Keeps bar background visible to prevent wallpaper flash
# ONLY active when bar is at top position

# Query current bar position - skip auto-hide when bar is at bottom
BAR_POSITION=$(sketchybar --query bar | grep -o '"position":"[^"]*"' | cut -d'"' -f4)
if [ "$BAR_POSITION" = "bottom" ]; then
    exit 0
fi

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
