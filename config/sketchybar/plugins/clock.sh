#!/bin/bash

# Clock plugin
# Shows current time and date

sketchybar --set $NAME label="$(date '+%a %-m/%-d %-l:%M')"
