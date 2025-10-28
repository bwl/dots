#!/bin/bash

# Space/Workspace indicator plugin
# Shows which desktop space is currently active

source "$HOME/.config/sketchybar/colors.sh"

if [ "$SELECTED" = "true" ]; then
    sketchybar --set $NAME \
        background.color=$SPACE_ACTIVE \
        icon.color=$BAR_COLOR \
        label.color=$BAR_COLOR
else
    sketchybar --set $NAME \
        background.color=$SPACE_BG \
        icon.color=$ICON_COLOR \
        label.color=$LABEL_COLOR
fi
