# vendor/

Cloned external tools built from source. Contents gitignored (not tracked).

## Convention

- Clone repos here: `vendor/<tool-name>/`
- Build and install binaries to `~/.local/bin/`
- Each tool manages its own updates via git pull

## Current Tools

- **tmuxifier** - Tmux session/window/layout manager
- **cleanup-cache** - macOS/Linux cache cleanup utility (binary: `tidyup`)

## Adding a Tool

```bash
cd ~/dotfiles/vendor
git clone https://github.com/user/tool.git
cd tool
# build per tool instructions
mv <binary> ~/.local/bin/
```
