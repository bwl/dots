# Agent Delegation Decision Tree

## Quick Reference

| Task Type | Agent | Command Pattern | Why |
|-----------|-------|-----------------|-----|
| Architectural planning | Codex | `leaf plan send "..."` | Unlimited context, deep analysis |
| Sequential complex implementation | Codex | `leaf plan send "..."` | Needs understanding of prior steps |
| Parallel independent tasks | Cliffy | `leaf do tasks --tasks-file` | Fast, concurrent execution |
| Quick file edits (<50 lines) | Claude | Direct Edit tool | No coordination overhead |
| User communication | Claude | Direct response | You understand context |
| Workflow coordination | Claude | Direct task management | Meta-level orchestration |

## Detailed Decision Criteria

### Use Codex (`leaf plan`) When:

**Architectural Planning**
```
Symptoms:
- Need to design system components and their interactions
- Multiple interconnected design decisions
- Requires understanding full codebase context
- Needs to evaluate tradeoffs and edge cases

Example:
User: "Design an observability layer for Leaf"
→ Codex reads everything, analyzes architecture, produces detailed plan

Action:
leaf plan send "Read leaf-cli-plan.txt and leaf.sh. Design complete
observability implementation with JSON schemas, directory structure,
build sequence, and integration contracts. Make all design decisions."
```

**Deep Implementation**
```
Symptoms:
- Sequential work where each step builds on previous
- Needs to understand existing code deeply
- Complex refactoring across multiple functions
- Requires reading files >500 lines

Example:
User: "Refactor cmd_plan to use interactive REPL instead of one-off exec"
→ Codex reads full context, designs approach, implements methodically

Action:
leaf plan send "Read NEXT-STEP-REPL-REFACTOR.md and leaf.sh. Implement
the REPL refactoring following the plan exactly. Test each step."
```

**Bug Investigation**
```
Symptoms:
- Silent failures or mysterious behavior
- Need to trace through code paths
- Requires examining multiple files
- May need to test hypotheses

Example:
User: "leaf plan exits silently without errors"
→ Codex reads code, finds permission denied due to empty project_dir

Action:
leaf plan send "leaf plan is exiting silently. Read leaf.sh cmd_plan
function and trace the execution. Figure out what's failing and why.
Propose a fix."
```

**Code Review & Analysis**
```
Symptoms:
- Need expert assessment of implementation
- Architecture evaluation required
- Looking for edge cases or design flaws
- Want comprehensive analysis

Example:
User: "Review Codex's REPL implementation for correctness"
→ Codex reads the implementation and evaluates it

Action:
leaf plan send "Review the REPL refactoring implementation in leaf.sh.
Check for edge cases, error handling, integration with existing commands.
Verify it matches the plan in NEXT-STEP-REPL-REFACTOR.md."
```

### Use Cliffy (`leaf do tasks`) When:

**Parallel Independent Tasks**
```
Symptoms:
- Multiple tasks with no dependencies between them
- Each task is well-specified
- Work can happen simultaneously
- Simple, concrete actions

Example:
User: "Run tests, check linting, validate schemas"
→ Three independent checks that can run concurrently

Action:
Create tasks.yml:
---
tasks:
  - title: "Run unit tests"
    instructions: "Run 'go test ./...' and report results"
  - title: "Check shell linting"
    instructions: "Run 'shellcheck leaf.sh' and report issues"
  - title: "Validate JSON schemas"
    instructions: "Validate all .json files against schemas"

leaf do tasks --tasks-file tasks.yml --max-concurrent 3
```

**Batch Code Reviews**
```
Symptoms:
- Multiple files need similar analysis
- Pattern checking across codebase
- Security audits
- Documentation reviews

Example:
User: "Check all bash functions for proper error handling"
→ Independent review of each function

Action:
Create review-tasks.yml with one task per function
leaf do tasks --tasks-file review-tasks.yml --max-concurrent 5
```

**Parallel Build Steps**
```
Symptoms:
- Build sequence has identified parallel streams
- Steps are independent
- Each has clear success criteria

Example:
From OBSERVABILITY-IMPLEMENTATION.md parallel streams:
- Bash CLI team (steps 9-11)
- Testing team (step 12 fixtures)
→ After Go binary exists, these can run simultaneously

Action:
Create cli-tasks.yml (steps 9-11)
Create test-tasks.yml (step 12)
leaf do tasks --tasks-file cli-tasks.yml &
leaf do tasks --tasks-file test-tasks.yml
```

### Use Claude (You) When:

**Quick Edits**
```
Symptoms:
- Change is <50 lines
- Straightforward modification
- No deep analysis needed
- File already in context

Example:
User: "Add a comment explaining the resolve_project_dir function"
→ Simple, localized change

Action:
Use Edit tool directly on leaf.sh
```

**User Communication**
```
Symptoms:
- Need to clarify requirements
- Explain what's happening
- Summarize specialist outputs
- Answer questions about workflow

Example:
User: "What's the status of the REPL refactoring?"
→ You check what Codex did and summarize

Action:
Capture Codex output, provide concise summary to user
```

**Workflow Coordination**
```
Symptoms:
- Deciding who should do what
- Sequencing work appropriately
- Monitoring progress across agents
- Synthesizing multi-agent outputs

Example:
User: "Implement the observability layer"
→ You break this into Codex planning + Cliffy execution + Codex sequential work

Action:
1. Codex designs architecture
2. You create task files from the plan
3. Cliffy handles parallel work
4. Codex handles sequential complex work
5. You monitor and report to user
```

**Meta-Level Decisions**
```
Symptoms:
- Choosing between approaches
- Scoping work
- Identifying next steps
- Risk assessment

Example:
User: "Should we build the observability layer now or later?"
→ Strategic decision requiring project context

Action:
Consider project state, user needs, dependencies
Make recommendation with rationale
```

## Delegation Patterns

### Pattern: Plan → Execute

```
Step 1: Codex Plans
leaf plan send "Design X with complete schemas, build sequence, contracts"
↓
Step 2: Review & Refine
Claude: Check for vagueness, demand concrete details if needed
↓
Step 3: Create Task Files
Claude: Break plan into parallelizable tasks (tasks.yml)
↓
Step 4: Delegate Execution
Cliffy: leaf do tasks --tasks-file parallel-tasks.yml
Codex: leaf plan send "Implement sequential steps 2-8 from plan"
↓
Step 5: Monitor & Synthesize
Claude: Use leaf status/peek/wait to track progress
Claude: Synthesize results for user
```

### Pattern: Investigate → Fix

```
Step 1: Codex Investigates
leaf plan send "X is broken. Read relevant code, trace execution, find root cause"
↓
Step 2: Codex Proposes Fix
Codex: Returns analysis + proposed solution
↓
Step 3: Review Proposal
Claude: Verify fix makes sense, no unintended consequences
↓
Step 4: Implement
If simple: Claude implements directly
If complex: Codex implements via leaf plan send
↓
Step 5: Test
Claude or Cliffy: Run tests to verify fix
```

### Pattern: Review → Iterate

```
Step 1: Codex Reviews
leaf plan send "Review X for correctness, edge cases, and design flaws"
↓
Step 2: Codex Produces Report
Codex: Detailed analysis with specific concerns
↓
Step 3: Decide Next Steps
Claude: Determine if issues need fixing
↓
Step 4: If Fixes Needed
Codex: "Implement fixes for concerns 1, 3, and 5 from your review"
or
Cliffy: Create task file with independent fixes
↓
Step 5: Verify
Claude: Check that concerns are addressed
```

## Context Management

### When You're Approaching Limits (60%+ context)

**STOP doing:**
- Reading large files
- Analyzing complex code
- Planning architectures
- Deep implementation

**START doing:**
- Delegating to Codex with comprehensive context
- Creating task files for Cliffy
- Monitoring agent outputs
- Summarizing results

**Example:**
```
Your context: 140k / 200k (70% used)
User: "Now add time tracking metrics to the observability layer"

WRONG: Read the implementation plan yourself and try to design it
RIGHT:
  leaf plan send "Read OBSERVABILITY-IMPLEMENTATION.md and add time
  tracking metrics layer. Define event schema, storage format, CLI
  commands, integration with existing observer. Follow same pattern of
  concrete schemas and build sequence."
```

### When Agent Has More Context

**Codex Advantages:**
- Unlimited context (can read entire codebase)
- Persistent memory within session
- Can reference all files without token cost

**Your Advantages:**
- Cross-session memory (via CLAUDE.md, plans, docs)
- User relationship understanding
- Workflow coordination ability
- Quick decision-making

**Implication:**
- Codex should do architecture and deep implementation
- You should do coordination and user interfacing

## Communication Patterns with Codex

### Effective Codex Instructions

**Structure:**
```
[Context paragraph 1]: What you need done and why
[Context paragraph 2]: Relevant file locations, constraints, requirements
[Context paragraph 3]: Expected deliverable format and success criteria
```

**Example:**
```
I need help refactoring the cmd_plan function. The current implementation
uses one-off 'codex exec' calls which re-contextualize every time. We want
to switch to an interactive REPL mode for persistent conversations.

Read leaf.sh focusing on cmd_plan (line 412+) and the helper functions
around line 180. Also read NEXT-STEP-REPL-REFACTOR.md which contains the
complete implementation plan. The plan specifies new helper functions, a
refactored cmd_plan, and a new cmd_do_codex for one-off execution.

Implement all the changes following the plan exactly. Make sure to use the
two-step tmux send-keys pattern (send text, then send C-m separately). Test
with bash -n leaf.sh when done.
```

### Ineffective Codex Instructions

**Too Vague:**
```
❌ "Fix the leaf plan function"
→ No context on what's wrong or what "fixed" means
```

**Too CLI-Like:**
```
❌ "codex exec refactor-plan.yml"
→ Treating Codex like a command-line tool, not explaining needs
```

**Too Minimal:**
```
❌ "Add error handling"
→ Where? What kind? What should it handle?
```

**Too Controlling:**
```
❌ "Use a hash map with linked list for LRU cache, implement it like..."
→ Micromanaging implementation details
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Doing It Yourself When Context-Limited

```
❌ WRONG:
Your context: 150k / 200k
Task: Design observability architecture
You: Start reading files, designing system, getting to 180k context

✅ RIGHT:
Your context: 150k / 200k
Task: Design observability architecture
You: leaf plan send "Read leaf-cli-plan.txt and leaf.sh. Design complete
     observability system with schemas and build sequence."
Codex: Has unlimited context, produces 269-line detailed plan
```

### Anti-Pattern 2: Using Codex for Simple Tasks

```
❌ WRONG:
Task: Add one-line comment to function
You: leaf plan send "Add a comment to function X explaining Y"
Codex: Spins up, reads file, adds comment (overhead ~30s)

✅ RIGHT:
Task: Add one-line comment to function
You: Use Edit tool directly (done in 2s)
```

### Anti-Pattern 3: Using Cliffy for Sequential Work

```
❌ WRONG:
Task: Refactor system where step 2 depends on step 1
You: Create tasks.yml with step 1 and step 2
Cliffy: Runs in parallel, step 2 fails because step 1 not done

✅ RIGHT:
Task: Refactor system where step 2 depends on step 1
You: leaf plan send "Implement steps 1-5 sequentially..."
Codex: Does steps in order, each building on previous
```

### Anti-Pattern 4: Serial Cliffy When Parallel Possible

```
❌ WRONG:
Tasks: Test A, Test B, Test C (all independent)
You: Run them one by one with separate leaf do tasks calls

✅ RIGHT:
Tasks: Test A, Test B, Test C (all independent)
You: Single tasks.yml with all three
    leaf do tasks --tasks-file tests.yml --max-concurrent 3
```

## Decision Flowchart

```
New task arrives
    ↓
Is it architectural planning or deep analysis?
    YES → Codex via leaf plan
    NO ↓
Is it >50 lines or requires reading large files?
    YES → Codex via leaf plan
    NO ↓
Are there multiple independent tasks?
    YES → Cliffy via leaf do tasks
    NO ↓
Is it simple edit in current context?
    YES → Claude handles directly
    NO ↓
Is it user communication or coordination?
    YES → Claude handles directly
    NO ↓
Default: Codex via leaf plan (when in doubt, delegate)
```

## Measuring Delegation Success

**Good Delegation Indicators:**
- Specialists produce detailed, actionable output
- Work happens in parallel when possible
- You stay below 70% context usage
- User gets timely, synthesized updates
- No rework due to misunderstanding

**Poor Delegation Indicators:**
- You're doing implementation at 80%+ context
- Specialists produce vague "consider this" responses
- Serial execution when parallel was possible
- User waiting for your bottlenecked work
- Rework due to insufficient initial specification

## Quick Reference Card

Keep this visible during coordination:

```
CONTEXT <60%           → You can handle simple tasks
CONTEXT 60-80%         → Delegate planning to Codex
CONTEXT >80%           → Delegate everything except summaries

TASK TYPE              → AGENT
Architecture           → Codex (leaf plan)
Sequential complex     → Codex (leaf plan)
Parallel independent   → Cliffy (leaf do tasks)
Simple edit           → Claude (direct)
User communication     → Claude (direct)
Coordination          → Claude (direct)

DELEGATION QUALITY
Good: Schemas + Build sequence + Contracts + Parallel streams identified
Bad: Vague descriptions + "Consider this" + No concrete deliverables
```
