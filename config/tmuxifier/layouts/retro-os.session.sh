#!/usr/bin/env bash
# RETRO-OS Session Layout
# Permanent tmux session with 3-pane interface

session_root "~/"
window_root "~/"

if initialize_session "retro-os"; then
  load_window "retro-os-main"
  select_window 0
fi

finalize_and_go_to_session
