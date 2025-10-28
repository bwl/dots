#!/bin/bash

# Battery plugin
# Shows battery percentage and charging status

source "$HOME/.config/sketchybar/colors.sh"
source "$HOME/.config/sketchybar/icons.sh"

PERCENTAGE=$(pmset -g batt | grep -Eo "\d+%" | cut -d% -f1)
CHARGING=$(pmset -g batt | grep 'AC Power')

if [ -n "$CHARGING" ]; then
    ICON=$ICON_BATTERY_CHARGING
    COLOR=$GREEN
else
    ICON=$ICON_BATTERY
    if [ $PERCENTAGE -lt 20 ]; then
        COLOR=$RED
    elif [ $PERCENTAGE -lt 50 ]; then
        COLOR=$YELLOW
    else
        COLOR=$GREEN
    fi
fi

sketchybar --set $NAME \
    icon="$ICON" \
    icon.color=$COLOR \
    label="${PERCENTAGE}%"
