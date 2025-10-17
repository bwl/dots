#!/bin/sh
#  shellcheck disable=SC2034,SC2154
#
#  Copyright (c) 2022-2023: Jacob.Lundqvist@gmail.com
#  License: MIT
#
#  Part of https://github.com/jaclu/tmux-menus
#
#  Main menu, the one popping up when you hit the trigger
#

static_content() {
    menu_name="Custom Menu"
    req_win_width=38
    req_win_height=26

    #  Menu items definition - Add your custom items here!
    set -- \
        0.0 S \
        0.0 T "--- Harpoon ---" \
        0.0 S \
        0.0 E h "Jump to tracked session" "run-shell '$HOME/.config/tmux/plugins/tmux-harpoon/harpoon -l'" \
        0.0 E a "Track current session" "run-shell '$HOME/.config/tmux/plugins/tmux-harpoon/harpoon -a'" \
        0.0 E A "Track current pane" "run-shell '$HOME/.config/tmux/plugins/tmux-harpoon/harpoon -A'" \
        0.0 E e "Edit tracked list" "run-shell '$HOME/.config/tmux/plugins/tmux-harpoon/harpoon -e'" \
        0.0 E d "Remove current session" "run-shell '$HOME/.config/tmux/plugins/tmux-harpoon/harpoon -d'" \
        0.0 S \
        0.0 T "--- Custom Actions ---" \
        0.0 S \
        0.0 E g "Open Ghostty" "open -a Ghostty" \
        0.0 C r "Reload tmux config" "source-file ~/.tmux.conf \; display-message 'Config reloaded!'" \
        0.0 S \
        0.0 T "--- Standard Menus ---" \
        0.0 S \
        0.0 M P "Handling Pane     -->" panes.sh \
        0.0 M W "Handling Window   -->" windows.sh \
        2.0 M S "Handling Sessions -->" sessions.sh \
        0.0 M L "Layouts           -->" layouts.sh \
        0.0 S \
        0.0 C n "Navigate & select ses/win/pane" "choose-tree"

    if tmux_vers_compare 2.7; then
        #  adds ignore case
        #  shellcheck disable=SC2145
        set -- "$@ -Z"
    fi

    set -- "$@" \
        0.0 T "-#[nodim]Search in all sessions & windows" \
        0.0 C s "only visible part"

    if tmux_vers_compare 3.2; then
        #  adds ignore case
        # shellcheck disable=SC2145
        set -- "$@, ignores case"
    fi

    set -- "$@" \
        "command-prompt -p 'Search for:' 'find-window"

    if tmux_vers_compare 3.2; then
        #  adds ignore case, and zooms the pane
        # shellcheck disable=SC2145
        set -- "$@ -Zi"
    fi

    #  shellcheck disable=SC2154
    set -- "$@" \
        0.0 S \
        0.0 E r 'Reload configuration file' reload_conf.sh \
        0.0 S \
        0.0 C d '<P> Detach from tmux' detach-client \
        0.0 S \
        0.0 M H 'Help -->' "$D_TM_ITEMS/help.sh $current_script"

    menu_generate_part 1 "$@"
}

#===============================================================
#
#   Main
#
#===============================================================

#  Full path to tmux-menus plugin
D_TM_BASE_PATH="$HOME/.config/tmux/plugins/tmux-menus"

#  Source dialog handling script
# shellcheck disable=SC1091
. "$D_TM_BASE_PATH"/scripts/dialog_handling.sh
