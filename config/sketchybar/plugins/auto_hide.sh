#!/bin/bash

# SketchyBar auto-hide plugin
# Responds to cursor position events from cursor_monitor.py
# Hides bar when cursor is near top edge (to reveal macOS menu bar)

case "$SENDER" in
  "cursor_at_top")
    # Cursor is near top edge - hide SketchyBar to reveal macOS menu bar
    sketchybar --bar hidden=on
    ;;
  "cursor_away_from_top")
    # Cursor moved away from top edge - show SketchyBar
    sketchybar --bar hidden=off
    ;;
esac
