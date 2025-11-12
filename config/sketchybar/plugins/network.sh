#!/bin/bash

# Network plugin
# Shows network connectivity status (WiFi SSID or Ethernet)

source "$HOME/.config/sketchybar/colors.sh"
source "$HOME/.config/sketchybar/icons.sh"

# Get WiFi SSID using system_profiler (more reliable)
SSID=$(system_profiler SPAirPortDataType 2>/dev/null | grep -A 1 "Current Network Information:" | grep -v "Current Network Information:" | grep -v "^--$" | head -1 | awk '{print $1}' | tr -d ':')

if [ -n "$SSID" ]; then
    # Connected to WiFi
    ICON=$ICON_WIFI
    #LABEL="$SSID"
    LABEL="Fios"
    COLOR=$CYAN
else
    # Check Ethernet fallback
    for interface in en0 en1 en2 en3 en4; do
        STATUS=$(ifconfig "$interface" 2>/dev/null | grep "status:" | awk '{print $2}')
        if [ "$STATUS" = "active" ]; then
            ICON=$ICON_ETHERNET
            LABEL="Ethernet"
            COLOR=$GREEN
            break
        fi
    done

    # No active connection found
    if [ -z "$LABEL" ]; then
        ICON=$ICON_WIFI_OFF
        LABEL="Offline"
        COLOR=$RED
    fi
fi

sketchybar --set $NAME \
    icon="$ICON" \
    icon.color=$COLOR \
    label="$LABEL"
