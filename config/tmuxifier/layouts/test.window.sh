# Test window layout for tmuxifier
window_root "~/"
new_window "test"

# Split into 2 panes for testing
split_h 50

# First pane
select_pane 0
run_cmd "echo 'Pane 1: tmuxifier working!'"

# Second pane
select_pane 1
run_cmd "echo 'Pane 2: layout test successful!'"

# Focus first pane
select_pane 0
