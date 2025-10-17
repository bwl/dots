#!/usr/bin/env bash
# RETRO-OS Main Window Layout
# 3-pane layout: sidebar (15%), workspace (60%), info panel (25%)

window_root "~/"
new_window "retro-os"

# Split into 3 panes: sidebar (15%), workspace (60%), info (25%)
split_h 25          # Right panel (25%)
select_pane 0
split_h 20          # Left sidebar (20% of remaining 75% ≈ 15% total)

# Set pane titles
tmux select-pane -t 0 -T "Sidebar"
tmux select-pane -t 1 -T "Workspace"
tmux select-pane -t 2 -T "Info"

# Start components
select_pane 0
run_cmd "sampler -c ~/.config/retro-os/sampler-sidebar.yml"

select_pane 1
run_cmd "~/bin/retro-os-dashboard"

select_pane 2
run_cmd "sampler -c ~/.config/retro-os/sampler-info.yml"

# Focus workspace
select_pane 1
