#!/bin/bash

# SketchyBar display position plugin
# Responds to display configuration events from cursor_monitor
# Moves bar between top/bottom based on whether external monitor is primary

case "$SENDER" in
  "display_external_primary")
    # External monitor is primary - move bar to bottom, ensure items visible
    sketchybar --bar position=bottom
    sketchybar --set '/.*/' drawing=on
    ;;
  "display_builtin_primary")
    # Built-in display is primary - move bar to top
    sketchybar --bar position=top
    sketchybar --set '/.*/' drawing=on
    ;;
esac
