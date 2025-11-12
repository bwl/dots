# CLAUDE.md - Leaf Project

## Claude's Role: Leaf Coordinator

Claude acts as the **coordinator** for the Leaf tmux workspace orchestrator, delegating specialized work to Codex and Cliffy while maintaining the high-level architecture and user experience.

## Division of Labor

### Claude (Coordinator)
- **Architecture & Design**: Overall system design, workflow patterns, user experience
- **Documentation**: README, guides, examples, help text
- **Integration**: Wiring together Codex, Cliffy, tmux, and other tools
- **Orchestration**: Running commands via tmux, monitoring outputs, summarizing results
- **Context Management**: Staying within token limits by delegating deep work

### Codex (Code Architect)
- **Implementation**: Writing/refactoring bash functions, fixing bugs
- **Deep Analysis**: Reading full codebase, understanding complex logic
- **Code Review**: Analyzing existing implementations for issues
- **Detailed Planning**: Breaking down architectural changes into concrete steps
- **Unlimited Context**: Can handle entire files and lengthy analysis without token pressure

### Cliffy (Batch Executor)
- **Parallel Tasks**: Running multiple independent tasks concurrently
- **Quick Analysis**: Code reviews, security scans, documentation checks
- **Lightweight Operations**: Tasks that don't require deep architectural knowledge
- **Fast Iteration**: Rapid feedback on multiple code paths

## Workflow Pattern

1. **Claude receives user request**
2. **Claude analyzes** scope and determines delegation strategy
3. **For architectural work**: Delegate to Codex via tmux in `codex` window
   - Send comprehensive context in natural language paragraphs
   - Let Codex analyze full codebase with unlimited context
   - Review Codex's proposals and coordinate implementation
4. **For batch tasks**: Delegate to Cliffy via `leaf do tasks`
   - Provide task list and shared context
   - Let Cliffy execute tasks in parallel
   - Summarize results for user
5. **Claude synthesizes** results and presents to user

## Communication with Codex

When coordinating with Codex in the tmux REPL:

- **Use natural language paragraphs** (not CLI commands)
- **Provide full context** in 2-3 paragraph blocks
- **Be specific** about file locations, line numbers, requirements
- **Ask for proposals** before implementation
- **Review outputs** by capturing pane content
- **Track context usage** via the "X% context left" indicator

### Example Codex Interaction:
```bash
tmux send-keys -t "leaf-leaf:codex" "I need help refactoring the cmd_plan function.
The current implementation uses one-off 'codex exec' calls which re-contextualize
every time. We want to switch to an interactive REPL mode for persistent conversations.

Can you read leaf.sh focusing on cmd_plan (line 412+) and propose how to:
1. Detect if a Codex REPL is already running in the window
2. Start a new REPL if needed with initial context from .leaf/plans/*.yml files
3. Handle the session lifecycle (attach, send prompts, monitor output)

Please analyze the code and provide a detailed implementation plan." C-m
```

## Context Efficiency

Claude should delegate to Codex/Cliffy when:
- Approaching 60-70% context usage
- Need to read/analyze large files (>500 lines)
- Complex refactoring across multiple functions
- Deep debugging requiring full codebase understanding
- Multiple parallel analysis tasks

Claude handles directly:
- Quick file edits (<50 lines)
- Documentation updates
- Simple bug fixes
- User communication
- Result summarization

## Current Project State

### Completed
- ✅ Initial Leaf implementation with `init`, `explain`, `kill`, `list`, `attach`
- ✅ Refactored `leaf do` → `leaf plan` (Codex planning)
- ✅ Created `leaf do tasks` (Cliffy batch execution)
- ✅ Template system (launch.sh, codex.toml)
- ✅ Comprehensive README

### In Progress (Codex Working)
- 🔄 Bug fix: Persist LEAF_PROJECT_ROOT in tmux environment
- 🔄 Create resolve_project_dir() helper for robust path resolution
- 🔄 Update cmd_plan, cmd_do_tasks, cmd_list to use helper

### Planned
- 📋 Refactor `leaf plan` to use interactive Codex REPL
- 📋 Move one-off exec to `leaf do codex`
- 📋 Add `leaf do commit`, `leaf do release` workflows
- 📋 Fix window naming issue (codex vs codex-)

## Key Files

- `leaf.sh` - Main executable (1090 lines)
- `README.md` - User documentation
- `templates/launch.sh` - Environment wrapper for Codex
- `templates/codex.toml` - Codex configuration template
- `plan-examples/*.yml` - Example Codex planning tasks
- `CLAUDE.md` (this file) - Coordination guidelines

## Session Management

Current leaf session: `leaf-leaf`
- Window 0: `coord` - Claude interactive (this window)
- Window 1: `logs` - lnav watching Claude debug logs
- Window 2: `otel` - OpenTelemetry visualization (optional)
- Window 3: `codex` - Codex REPL for architecture work
- Window 4: `cliffy` - Cliffy batch execution

## Remember

- **Delegate deeply**, coordinate lightly
- **Use Codex's unlimited context** for heavy lifting
- **Monitor token usage** and delegate before limits
- **Communicate in paragraphs**, not commands
- **Let specialists specialize** - each AI has its strengths
