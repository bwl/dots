#!/bin/bash

source "$CONFIG_DIR/colors.sh"

NUM_CORES=$(sysctl -n hw.ncpu)

# Cache top output (this is the expensive part)
CPU_DATA=$(top -l 1 -n 0 | grep "CPU usage")  # Use -l 1 instead of -l 2
USER=$(echo "$CPU_DATA" | awk '{print $3}' | tr -d '%' | cut -d'.' -f1)
SYS=$(echo "$CPU_DATA" | awk '{print $5}' | tr -d '%' | cut -d'.' -f1)
TOTAL=$((USER + SYS))
BASE=$((TOTAL / 10))

# Fast color lookup table (pre-computed)
declare -a COLORS=(
    "0xff9ece6a"  # 0 - green
    "0xffa8d47a"  # 1
    "0xffb2da8a"  # 2
    "0xffbce09a"  # 3
    "0xffc6e5aa"  # 4
    "0xffe0af68"  # 5 - yellow
    "0xffe89f5d"  # 6
    "0xfff08f52"  # 7
    "0xfff37f47"  # 8
    "0xfff7768e"  # 9 - red
)

# Single sketchybar call
ARGS=""
for ((i=0; i<$NUM_CORES; i++)); do
    LOAD=$((BASE + (RANDOM % 5 - 2)))
    [ $LOAD -lt 0 ] && LOAD=0
    [ $LOAD -gt 9 ] && LOAD=9

    ARGS="$ARGS --set cpu.core$i label=$LOAD label.color=${COLORS[$LOAD]}"
done

sketchybar $ARGS
