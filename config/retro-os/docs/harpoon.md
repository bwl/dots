# RETRO-OS: Harpoon Bookmarks

Tmux-harpoon lets you bookmark frequently-used sessions for quick access.

## Keybindings

- **Ctrl+H**: Fuzzy jump to bookmarked session
- **Ctrl+Shift+H**: Bookmark current session

## Usage

1. Navigate to a session you use often (e.g., RETRO-OS)
2. Press **Ctrl+Shift+H** to bookmark it in slot 1
3. Navigate to another session (e.g., dotfiles fresh)
4. Press **Ctrl+Shift+H** to bookmark it in slot 2
5. Press **Ctrl+H 1** to jump to slot 1
6. Press **Ctrl+H 2** to jump to slot 2

## Fuzzy Jump

Press **Ctrl+H** (no number) to see a fuzzy list of bookmarked sessions.

## Storage

Bookmarks are stored in `~/.tmux-harpoon.json`
