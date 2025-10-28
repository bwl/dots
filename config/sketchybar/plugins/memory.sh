#!/bin/bash

# Memory usage plugin
# Shows current RAM utilization

source "$HOME/.config/sketchybar/colors.sh"

# Get memory pressure (using vm_stat for better accuracy)
MEMORY_PRESSURE=$(memory_pressure | grep "System-wide memory free percentage:" | awk '{print 100-$5}' | sed 's/%//')

# Fallback to simpler method if memory_pressure fails
if [ -z "$MEMORY_PRESSURE" ]; then
    MEMORY_USED=$(vm_stat | grep "Pages active" | awk '{print $3}' | sed 's/\.//')
    MEMORY_FREE=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
    MEMORY_TOTAL=$((MEMORY_USED + MEMORY_FREE))
    MEMORY_PRESSURE=$((MEMORY_USED * 100 / MEMORY_TOTAL))
fi

# Color based on usage
if [ $MEMORY_PRESSURE -gt 80 ]; then
    COLOR=$RED
elif [ $MEMORY_PRESSURE -gt 60 ]; then
    COLOR=$YELLOW
else
    COLOR=$GREEN
fi

sketchybar --set $NAME \
    label="${MEMORY_PRESSURE}%" \
    label.color=$COLOR
