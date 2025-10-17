# Fresh Enhancement Ideas

Ideas for expanding the `fresh` command into a comprehensive AI-assisted development environment.

---

## 1. Homescreen Dashboard

**Concept**: A dedicated window (index 0) that shows an at-a-glance overview of your project.

**Features**:
- Project name and git status summary
- Recently modified files
- Active tasks from taskbook
- Quick links to documentation
- System resource usage (CPU, memory, disk)
- Last commit info and branch status

**Implementation**:
- Use `tmux-sidebar` or custom script with `watch` command
- Split panes: top half shows project info, bottom half shows quick actions menu
- Auto-refresh every 5 seconds

---

## 2. File Navigator Window

**Concept**: Persistent file browser using [tmux-sidebar](https://github.com/tmux-plugins/tmux-sidebar) or similar.

**Features**:
- Tree-view file browser in sidebar (already configured in tmux.conf)
- Integration with `fzf` for fuzzy file search
- Quick preview pane for selected files
- Git status indicators in file list

**Current Status**:
- tmux-sidebar already installed as plugin
- Trigger with `prefix` + `Tab` or `prefix` + `Backspace`

**Enhancements**:
- Add dedicated window with full-screen file explorer using `ranger` or `lf`
- Integrate with ripgrep for content search
- Show file metadata and git blame info

---

## 3. Enhanced Command Palette

**Concept**: Context-aware command launcher inspired by VSCode's command palette and Raycast.

**Features**:
- Fuzzy searchable list of all available actions
- Recent commands history
- Project-specific shortcuts
- Agent-aware commands (send prompts to specific agents)
- Git operations menu
- Session management commands

**Implementation**:
- Extend custom-menu.sh with fzf-powered interface
- Bind to `Ctrl-Space` or custom key
- Commands include:
  - "Ask Claude to..." → Send prompt to claude window
  - "Search codebase with Codex..."
  - "Add task..." → Create task in tb
  - "Git commit..." → Jump to lazygit
  - "Kill session"
  - "Explain current window" → Run `fresh explain`

**Tech Stack**: tmux-fzf plugin (already installed) + custom shell scripts

---

## 4. Settings/Config Window

**Concept**: Interactive configuration manager for fresh sessions.

**Features**:
- View and edit environment variables (TMUX_*_CMD overrides)
- Toggle auto-refresh intervals
- Configure which windows to create by default
- Set project-specific preferences (stored in `.fresh.yaml` or similar)
- Theme switching
- Plugin management

**Implementation**:
- `fresh config` subcommand
- Interactive TUI using `dialog`, `whiptail`, or custom fzf menus
- Save project configs to `.fresh/config.yaml`
- Global defaults in `~/.config/fresh/config.yaml`

---

## 5. AI Agent Orchestration Window

**Concept**: Control center for coordinating multiple AI agents on complex tasks.

**Features**:
- Visual representation of all active agents and their status
- Task queue showing what each agent is working on
- Ability to send coordinated prompts to multiple agents
- View agent conversation history
- Agent-to-agent communication examples
- Pre-built workflows (e.g., "Refactor → Test → Document")

**Implementation**:
- New window with split layout
- Left pane: agent status dashboard
- Right pane: workflow controls
- Use tmux `send-keys` and `capture-pane` for orchestration

---

## 6. Real-time Logs/Monitoring Window

**Concept**: Aggregated log viewer for development servers, test runners, and background processes.

**Features**:
- Tail multiple log files in split panes
- Colored output with pattern highlighting
- Filter logs by severity or pattern
- Watch test output in real-time
- Monitor Docker containers or systemd services
- Alert on error patterns

**Implementation**:
- Use `multitail`, `lnav`, or custom `tail -f` with `awk`/`grep` filtering
- Split window into multiple log panes
- Configure log sources per-project

---

## 7. Documentation Browser Window

**Concept**: Quick-access documentation without leaving the terminal.

**Features**:
- Project README.md viewer with live reload
- Man pages and cheat sheets
- Language/framework documentation (using `cheat`, `tldr`, `devdocs-cli`)
- API reference browser
- Project wiki or notes

**Implementation**:
- Use `glow` or `mdcat` for markdown rendering
- `cheat.sh` integration for quick reference
- fzf menu to select documentation to view
- Could use `w3m` or `lynx` for web docs

---

## 8. Testing & CI Window

**Concept**: Dedicated environment for running and monitoring tests.

**Features**:
- Run test suites with watch mode
- Show test coverage reports
- CI/CD pipeline status
- Test failure navigator (jump to failing tests)
- Quick re-run of failed tests
- Benchmark comparison view

**Implementation**:
- Integration with `pytest --watch`, `jest --watch`, `cargo watch`, etc.
- Parse test output and create interactive failure list
- Use GitHub Actions or similar API to show CI status
- Could replace or complement `cliffy` window

---

## 9. Snippet & Template Manager

**Concept**: Quick access to code snippets and project templates.

**Features**:
- Search and insert code snippets
- Project scaffolding templates
- Boilerplate generation
- Common git commit message templates
- Prompt templates for AI agents
- Shell command history with descriptions

**Implementation**:
- fzf-powered snippet browser
- Store snippets in `~/.config/fresh/snippets/` or `.fresh/snippets/`
- Use `pet` (snippet manager) or custom solution
- Integrate with tmux copy mode to insert snippets

---

## 10. Project Switcher

**Concept**: Fast switching between multiple project sessions.

**Features**:
- List all active fresh sessions
- Recently used projects
- Favorite/pinned projects
- Create new session in any directory
- Preview project info before switching
- Bulk session management (kill multiple, rename, etc.)

**Implementation**:
- `fresh switch` command with fzf menu
- Integration with `tmux list-sessions`
- Store project metadata in `~/.config/fresh/recent-projects.json`
- Could use tmux-sessionx plugin or similar

---

## 11. Git Workflow Assistant

**Concept**: Enhanced git integration beyond just lazygit.

**Features**:
- Conventional commit message builder
- PR creation workflow (title, description template, labels)
- Branch name generator following conventions
- Interactive rebase helper
- Merge conflict resolver
- Stash manager with previews

**Implementation**:
- Scripts using `gh` CLI and `git` commands
- TUI built with `dialog` or `gum`
- Integration with existing git window
- Pre-commit hook integration

---

## 12. Performance Profiler Window

**Concept**: Monitor and profile application performance.

**Features**:
- CPU/memory profiling of running processes
- Request/response time monitoring
- Database query analyzer
- Flame graph visualization (using `flamegraph.pl` or similar)
- Load testing results
- Bundle size analyzer for web projects

**Implementation**:
- Integration with language-specific profilers
- `htop`/`btop` for system monitoring
- Custom parsers for profiler output
- Could use `hyperfine` for benchmarking

---

## 13. Database Explorer

**Concept**: Interactive database client within tmux.

**Features**:
- Query builder and executor
- Schema browser
- Table data viewer with pagination
- Query history
- Connection manager for multiple databases
- Export query results

**Implementation**:
- Use `usql` (universal SQL CLI) or `mycli`/`pgcli`
- fzf menus for table/column selection
- Could integrate with Datagrip or similar

---

## 14. Notification Center

**Concept**: Centralized alerts and notifications from all windows.

**Features**:
- Test failure notifications
- CI/CD status updates
- Long-running command completion alerts
- Agent task completion notifications
- System alerts (disk space, etc.)
- Custom user-defined triggers

**Implementation**:
- Status bar integration using catppuccin-tmux
- Desktop notifications using `terminal-notifier` (macOS) or `notify-send` (Linux)
- tmux message area for in-terminal notifications
- Integration with tmux-prefix-highlight plugin

---

## 15. Collaborative Session Sharing

**Concept**: Share your fresh session with teammates or AI assistants.

**Features**:
- Generate shareable session via `tmate` or `tmux-share`
- Read-only observer mode
- Screen recording/replay of session
- Export session history as Markdown
- Share specific window outputs

**Implementation**:
- `tmate` for live session sharing
- `asciinema` for recording
- Custom export script to save all pane outputs
- Integration with GitHub Gists or similar

---

## Implementation Priority

**Phase 1 (Quick Wins)**:
1. Enhanced Command Palette (#3)
2. Project Switcher (#10)
3. Settings Window (#4)

**Phase 2 (Core Features)**:
4. Homescreen Dashboard (#1)
5. AI Agent Orchestration (#5)
6. Documentation Browser (#7)

**Phase 3 (Advanced Features)**:
7. Testing & CI Window (#8)
8. Notification Center (#14)
9. Git Workflow Assistant (#11)

**Phase 4 (Specialized)**:
10. Real-time Logs (#6)
11. Snippet Manager (#9)
12. Remaining features as needed

---

## Configuration Design

Each enhancement should support:
- **Global defaults**: `~/.config/fresh/config.yaml`
- **Project overrides**: `.fresh/config.yaml` in project root
- **Environment variables**: `FRESH_*` prefix for runtime overrides
- **Command-line flags**: `fresh --enable-dashboard`, etc.

Example config structure:
```yaml
windows:
  dashboard:
    enabled: true
    position: 0
    refresh_interval: 5

  navigator:
    enabled: true
    tool: "ranger"  # or "lf", "nnn"

  agents:
    claude:
      command: "claude"
      sidebar: true
    codex:
      command: "codex"
      sidebar: true

features:
  command_palette_key: "C-Space"
  notification_center: true
  auto_commit_messages: true

theme: "catppuccin-mocha"
```

---

## Notes

- Keep backwards compatibility with existing `fresh` command
- All enhancements should be opt-in via configuration
- Focus on keyboard-first, terminal-native UX
- Leverage existing tmux plugins where possible
- Document agent capabilities in each window via `fresh explain`
