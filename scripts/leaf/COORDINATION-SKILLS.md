# Coordination Skills - Reusable Patterns

These are executable coordination patterns. When you recognize a situation, follow the corresponding skill.

## Skill: Architectural Planning Session

**When to use:** User requests new feature or system that requires design work

**Steps:**
1. Clarify scope and requirements with user (ask questions if needed)
2. Identify what Codex needs to read to understand context
3. Craft 2-3 paragraph brief for Codex including:
   - What needs to be designed and why
   - Relevant file locations and existing patterns
   - Required deliverables (schemas, build sequence, contracts)
   - Concrete decisions needed (no "consider" allowed)
4. Send via `leaf plan send` (or tmux send-keys if REPL running)
5. Wait for Codex to complete
6. Review output for concreteness
7. If vague → Push back with specific requirements
8. If concrete → Present summary to user

**Example:**
```
User: "Add metrics tracking to Leaf"

1. Ask: "What kind of metrics? Execution times? Success rates? Both?"
User: "Execution times and anomaly detection"

2. Identify context: leaf.sh, existing .leaf/ directory structure, OBSERVABILITY-IMPLEMENTATION.md

3. Craft brief:
"I need you to design a metrics tracking and anomaly detection layer for Leaf.

Read OBSERVABILITY-IMPLEMENTATION.md which has the observability architecture.
The metrics layer should track execution times for tasks, compute historical
averages, detect anomalies (tasks running longer than expected), and provide
time estimates for in-progress work.

Create METRICS-LAYER.md with:
- Event schema for task.start / task.finish
- Storage format (JSONL or SQLite)
- Calculation methods (EWMA for averages, 2×expected for anomalies)
- CLI commands (leaf estimate, leaf history, leaf stats)
- Integration points with existing leaf commands
All schemas defined, all contracts specified, build sequence dependency-ordered."

4. leaf plan send [brief]

5. Wait for completion

6. Review: Does it have complete JSON schemas? Build sequence? Exit codes?

7a. If vague: "The storage format says 'JSONL or SQLite' - commit to one.
               Define the exact JSONL schema if that's the choice."

7b. If concrete: "Codex designed metrics layer with JSONL event logs,
                  EWMA-based predictions, and anomaly detection at 2×expected.
                  Ready to implement."
```

## Skill: Plan → Execute Workflow

**When to use:** After Codex produces detailed implementation plan

**Steps:**
1. Read the plan Codex produced
2. Identify build sequence and dependencies
3. Separate into:
   - Sequential work (must be done in order)
   - Parallel work (independent tasks)
4. Create task files for parallel work
5. Launch appropriate agents:
   - Codex for sequential complex work
   - Cliffy for parallel independent work
6. Monitor progress via leaf status/peek
7. Synthesize results for user

**Example:**
```
Codex produced: OBSERVABILITY-IMPLEMENTATION.md
Build sequence has:
- Steps 2-8: Sequential Go backend work
- Steps 9-11: Bash CLI (can start after step 8)
- Step 12: Test fixtures (can start after step 4)

1. Read the plan ✓

2. Identified sequence:
   - 2→3→4→5→6→7→8 (sequential)
   - 4→12 (parallel branch)
   - 8→9→10→11 (parallel branch)

3. Separate:
   Sequential: Steps 2-8 (Go backend)
   Parallel A: Step 12 (test fixtures, depends on 4)
   Parallel B: Steps 9-11 (Bash CLI, depends on 8)

4. Create test-fixtures-tasks.yml:
---
tasks:
  - title: "Create codex test captures"
    instructions: "Create testdata/captures/codex/{ready,working,failed}.txt"
  - title: "Create event log samples"
    instructions: "Create testdata/events/sample.jsonl with test events"

5. Launch:
   # Start sequential work
   leaf plan send "Implement steps 2-8 from OBSERVABILITY-IMPLEMENTATION.md.
   Go backend work: capture.go, config.go, classify.go, cache.go, events.go,
   model.go, and CLI binary main.go. Test each module as you build."

   # Monitor until step 4 done, then launch parallel test work
   leaf wait codex done  # (will support this soon)
   # Check logs that step 4 is complete
   leaf do tasks --tasks-file test-fixtures-tasks.yml

   # When step 8 done, launch bash CLI work
   # (Could use Cliffy for steps 9-11 if simple enough)

6. Monitor:
   leaf status codex     # Check Go backend progress
   leaf status cliffy    # Check test fixture progress

7. Synthesize:
   "Codex completed Go backend (steps 2-8). Cliffy created test fixtures.
    Ready for Bash CLI integration (steps 9-11)."
```

## Skill: Iterative Refinement

**When to use:** Codex produces output that needs to be more concrete

**Steps:**
1. Review output for vagueness
2. Identify specific gaps:
   - Missing schemas
   - Unspecified contracts
   - "Consider" instead of decisions
   - Vague error handling
3. Craft precise follow-up
4. Send to Codex
5. Repeat until concrete

**Example:**
```
Codex produces observability plan mentioning "State detection via heuristics"

1. Review: This is vague

2. Identify gaps:
   - No heuristic configuration format
   - No confidence threshold specified
   - "Consider Go or Bash" is not a decision

3. Craft follow-up:
"Good architectural overview. Now make it actionable:

1. Commit to Go for state detection. No more 'consider' - just use Go.
2. Define the exact heuristic configuration JSON schema.
3. Specify confidence thresholds: what's high/medium/low confidence?
4. Define all exit codes the binary will use.
5. Specify the integration contract: exact command-line flags,
   environment variables, and output format."

4. Send via leaf plan send

5. Codex produces OBSERVABILITY-IMPLEMENTATION.md with all details

   If still vague → repeat
   If concrete → approve
```

## Skill: Bug Investigation

**When to use:** Something is broken and needs diagnosis

**Steps:**
1. Gather symptoms from user
2. Identify relevant code areas
3. Brief Codex with symptoms + code locations
4. Codex investigates and proposes fix
5. Review fix for correctness
6. Implement fix (Codex or you, depending on complexity)
7. Test and confirm

**Example:**
```
User: "leaf plan exits silently"

1. Symptoms: No error message, no output, just exits

2. Relevant: cmd_plan function in leaf.sh, around line 412+

3. Brief Codex:
"leaf plan is exiting silently without any error messages.

Read leaf.sh focusing on cmd_plan (line 412+) and trace the execution path.
The function should create a .leaf/plans directory and generate a run script,
but it's failing silently.

Likely causes:
- Directory creation failing
- Script generation failing
- set -euo pipefail causing exit on error without message

Find the root cause and propose a fix that includes proper error handling."

4. Codex investigates → finds empty project_dir causing mkdir -p /.leaf/

5. Review fix: Makes sense, add resolve_project_dir() helper

6. Codex implements the helper function and updates callers

7. Test: leaf plan now works correctly
```

## Skill: Parallel Task Decomposition

**When to use:** Independent work items can run simultaneously

**Steps:**
1. Identify independent tasks
2. Verify no cross-dependencies
3. Create tasks.yml with concrete specifications
4. Set appropriate concurrency limit
5. Launch via leaf do tasks
6. Monitor completion
7. Collect and summarize results

**Example:**
```
User: "Audit the codebase for security issues"

1. Independent tasks:
   - Check for hardcoded secrets
   - Validate input sanitization
   - Review authentication logic
   - Check for command injection risks
   - Audit file permission handling

2. Verify independence: ✓ Each can be done without others

3. Create security-audit-tasks.yml:
---
tasks:
  - title: "Check for hardcoded secrets"
    instructions: "Scan all .sh files for API keys, passwords, tokens.
                   Check for patterns like: PASSWORD=, API_KEY=, secret=.
                   Report any findings with file:line locations."

  - title: "Validate input sanitization"
    instructions: "Review all user input handling in leaf.sh.
                   Check that variables are quoted, inputs validated.
                   Report any unsafe variable expansions."

  - title: "Review authentication logic"
    instructions: "Check how leaf verifies tmux session ownership.
                   Ensure no privilege escalation vectors.
                   Report any concerns."

  - title: "Check for command injection"
    instructions: "Review all system command executions.
                   Look for unquoted variables in commands.
                   Check for proper escaping. Report issues."

  - title: "Audit file permissions"
    instructions: "Review file creation in .leaf/ directory.
                   Check that sensitive files have proper permissions.
                   Report any world-readable sensitive files."

4. Concurrency: 5 tasks, set --max-concurrent 3

5. Launch:
   leaf do tasks --tasks-file security-audit-tasks.yml --max-concurrent 3

6. Monitor:
   leaf status cliffy  # Check progress

7. Collect results:
   "Security audit complete. Cliffy found:
    - No hardcoded secrets
    - 2 unquoted variables in line 312, 445
    - Authentication logic is sound
    - No command injection vectors
    - File permissions need tightening on .leaf/cache/"
```

## Skill: Context-Aware Delegation

**When to use:** Approaching context limits while working

**Steps:**
1. Check context usage
2. If >60% → Stop what you're doing
3. Identify remaining work
4. Package context for Codex
5. Delegate with comprehensive brief
6. Clear your context by focusing on coordination

**Example:**
```
Current context: 140k / 200k (70%)
Task: Implement metrics layer with schemas, storage, CLI commands

1. Context check: 70% - need to delegate

2. Stop: Don't try to design schemas yourself

3. Remaining work:
   - Define event schemas
   - Design storage format
   - Specify CLI commands
   - Create integration points

4. Package:
   "Files to read: OBSERVABILITY-IMPLEMENTATION.md, leaf.sh
    Requirement: Metrics layer for execution time tracking
    Deliverables: Event schema, storage design, CLI specs, integration"

5. Delegate:
   leaf plan send "Design metrics layer...
   [comprehensive brief with all context]"

6. Focus on coordination:
   - Monitor Codex progress
   - Prepare to review output
   - Stay available for user questions
   - Keep context light
```

## Skill: Multi-Agent Synthesis

**When to use:** Multiple agents worked on different parts, need unified result

**Steps:**
1. Collect outputs from all agents
2. Verify each part is complete
3. Check for integration points
4. Identify any gaps or conflicts
5. Resolve issues (coordinate with agents if needed)
6. Create unified summary for user
7. Identify next steps

**Example:**
```
Codex completed: Go backend (steps 2-8)
Cliffy completed: Test fixtures (step 12)
Codex completed: Bash CLI (steps 9-11)

1. Collect:
   - Go binary built: cmd/leaf-detect-state
   - Test captures created: testdata/captures/
   - Bash wrapper created: lib/detect.sh
   - Commands added: leaf status, leaf peek, leaf wait

2. Verify completeness:
   ✓ Binary exists and compiles
   ✓ Test fixtures have all states
   ✓ Wrapper handles all exit codes
   ✓ Commands integrated into leaf.sh

3. Integration points:
   - Binary called by wrapper ✓
   - Wrapper called by commands ✓
   - Test fixtures used by tests ✓

4. Gaps:
   - Tests not yet run
   - Documentation not updated

5. Resolve:
   - Run test suite
   - Update README with new commands

6. Unified summary:
   "Observability layer implementation complete:
    - Go state detector binary (8 modules, tested)
    - Bash CLI commands (status/peek/wait)
    - Test fixtures covering all agent states

    Remaining: Update README with usage examples"

7. Next steps:
   - Run integration tests
   - Update documentation
   - Test in live leaf session
```

## Skill: Pushing Back on Vagueness

**When to use:** Agent produces non-actionable output

**Steps:**
1. Identify vague elements
2. List specific requirements
3. Demand concrete answers
4. Give examples of what "concrete" means
5. Don't accept hand-waving

**Example:**
```
Codex produces: "Use a cache with TTL-based invalidation"

1. Vague elements:
   - What's the cache data structure?
   - What's the TTL value?
   - Where is the cache stored?
   - What triggers invalidation besides TTL?

2. Specific requirements:
   "I need concrete cache specifications:

   1. Define the exact JSON schema for cache entries
   2. Specify the TTL in seconds (not 'reasonable' - actual number)
   3. Specify the file path (.leaf/cache/state-cache.json?)
   4. Define all invalidation triggers:
      - TTL expiration
      - Content hash changes?
      - Send-keys events?
      - Manual --force flag?
   5. Specify cache size limits (max entries? max file size?)"

3. Demand:
   "Don't say 'consider' or 'probably' or 'might'. Make decisions.
    If you need more context to decide, tell me what you need."

4. Give example:
   "Good: 'Cache TTL is 1 second, stored at .leaf/cache/state-cache.json'
    Bad: 'Cache should have a reasonable TTL'"

5. Don't accept:
   ❌ "The cache could use LRU eviction..."
   ✅ "The cache uses LRU eviction with max 100 entries."
```

## Skill: Status Check and Report

**When to use:** User asks for progress update

**Steps:**
1. Check all active agents
2. Read recent outputs
3. Assess state and progress
4. Estimate proximity to completion (based on work done, not time)
5. Identify blockers if any
6. Provide concise summary

**Example:**
```
User: "What's the status of the observability implementation?"

1. Check agents:
   leaf status codex  → "working" (implementing Go backend)
   leaf status cliffy → "ready" (completed test fixtures)

2. Read outputs:
   leaf peek codex -n 30
   → See: "Implemented classify.go, starting cache.go"

3. Assess progress:
   Plan has 8 steps for Go backend
   Completed: steps 2, 3, 4 (capture, config, classify)
   Current: step 5 (cache)
   Remaining: steps 6, 7, 8 (events, model, binary)

4. Proximity: 50% complete (4/8 steps)

5. Blockers: None visible

6. Summary:
   "Observability implementation in progress:

    Codex: 50% through Go backend (4/8 steps)
    - ✅ Capture abstraction
    - ✅ Config loader
    - ✅ Heuristic classifier
    - 🔄 Cache subsystem (current)
    - ⏳ Events module
    - ⏳ Observer model
    - ⏳ CLI binary

    Cliffy: ✅ Test fixtures complete

    Next: Once Go backend done, integrate Bash CLI"
```

## Skill: Emergency Context Management

**When to use:** Context at 85%+ and can't delegate immediately

**Steps:**
1. Stop all analysis and reading
2. Save state to file if needed
3. Summarize what's done
4. Identify absolute minimum to complete
5. Complete only that minimum
6. Hand off to Codex for continuation

**Example:**
```
Context: 175k / 200k (87%)
Situation: Mid-implementation of cache module

1. STOP: Don't read any more files

2. Save state:
   Create CACHE-WIP.md:
   "Cache module partially implemented.
    Done: JSON schema, file read/write
    TODO: TTL checking, hash comparison, invalidation
    File: internal/observer/cache.go:1-145"

3. Summarize:
   "Cache module 60% done. Read/write working, need TTL logic."

4. Minimum:
   - Finish current function (not whole module)
   - Make it syntactically valid
   - Commit work-in-progress

5. Complete:
   - Close the open function
   - Add TODO comments for remaining work
   - Verify builds

6. Hand off:
   leaf plan send "Continue cache.go implementation.
   Read CACHE-WIP.md for current state.
   Complete TTL checking, hash comparison, and invalidation logic.
   Follow the schema in OBSERVABILITY-IMPLEMENTATION.md."
```

## Quick Reference

```
SITUATION                           → SKILL
User wants new feature              → Architectural Planning Session
Codex finished detailed plan        → Plan → Execute Workflow
Output is vague                     → Iterative Refinement
Something is broken                 → Bug Investigation
Multiple independent tasks          → Parallel Task Decomposition
Context >60%                        → Context-Aware Delegation
Multiple agents finished            → Multi-Agent Synthesis
Agent being non-specific            → Pushing Back on Vagueness
User asks for status                → Status Check and Report
Context >85%                        → Emergency Context Management
```
