# RETRO-OS: Troubleshooting

## Session Not Loading

**Problem**: `retro-os` command shows plain shell, no 3-pane layout.

**Solution**:
```bash
# Kill broken session
tmux kill-session -t retro-os

# Reload shell config
exec zsh

# Boot fresh
tmuxifier load-session retro-os
```

## Ctrl+Space Not Working

**Check**: `~/.config/tmux/custom-menu.sh` path is correct
**Solution**: `tmux source-file ~/.tmux.conf`

## Pod Won't Spawn

**Check error log**: `tail ~/.local/state/retro-os/error.log`
**Common issues**:
- Command not installed (install via brew)
- Permissions (chmod +x ~/bin/pods/*.sh)

## Sampler Not Refreshing

**Solution**: Kill sampler panes and restart layout
```bash
tmux kill-session -t retro-os
tmuxifier load-session retro-os
```

## Missing Dependencies

Run health check: Ctrl+Space → "admin" → option 2
