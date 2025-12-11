#!/bin/bash
# analyze-project-deep.sh - Use Claude CLI to generate deep analysis content
# Usage: analyze-project-deep.sh <project-path> [--summary-only]
#
# This script:
# 1. Uses Claude CLI with read-only tools to analyze the project
# 2. Generates a complete analysis file with filled-in content

set -uo pipefail

PROJECT_PATH="$1"
SUMMARY_ONLY="${2:-}"

if [[ -z "$PROJECT_PATH" ]]; then
    echo "Usage: analyze-project-deep.sh <project-path> [--summary-only]"
    exit 1
fi

if [[ ! -d "$PROJECT_PATH" ]]; then
    echo "Error: Directory not found: $PROJECT_PATH"
    exit 1
fi

PROJECT_NAME=$(basename "$PROJECT_PATH")
OUTPUT_DIR="$HOME/Developer/ideas/_data/analysis"
OUTPUT_FILE="$OUTPUT_DIR/$PROJECT_NAME.md"
TODAY=$(date +%Y-%m-%d)

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

echo "=== Deep Analysis: $PROJECT_NAME ==="
echo ""

# Build the prompt - ask Claude to generate the entire markdown file
PROMPT="Analyze the project at: $PROJECT_PATH

Explore the codebase using Read, Glob, and Grep tools to understand its architecture.

Then output a JSON object with these fields:
- tagline: One sentence describing what the project does
- type: Project type (e.g., \"Rust CLI\", \"Bevy game\", \"TypeScript app\")
- status: Activity level (\"Active development\", \"Maintained\", \"Early development\", \"Stale\")
- tech: Comma-separated tech stack (e.g., \"rust, bevy, ratatui\")
- entry_point: Main entry file (e.g., \"src/main.rs\")
- what_it_does: 2-3 paragraphs explaining purpose, features, current state
- architecture: Bullet points of main modules and their responsibilities
- key_files: Array of {file, purpose} for important files
- module_breakdown: Detailed analysis of each module (skip if summary-only)
- design_patterns: Notable patterns in the codebase (skip if summary-only)
- strengths: Code quality strengths (skip if summary-only)
- improvements: Areas for improvement (skip if summary-only)
- relationships: How this relates to other projects (skip if summary-only)
- open_questions: Things needing investigation (skip if summary-only)

Be specific to THIS codebase. Read actual source files before answering."

# JSON schema for structured output
SCHEMA='{
  "type": "object",
  "properties": {
    "tagline": {"type": "string"},
    "type": {"type": "string"},
    "status": {"type": "string"},
    "tech": {"type": "string"},
    "entry_point": {"type": "string"},
    "what_it_does": {"type": "string"},
    "architecture": {"type": "string"},
    "key_files": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "file": {"type": "string"},
          "purpose": {"type": "string"}
        },
        "required": ["file", "purpose"]
      }
    },
    "module_breakdown": {"type": "string"},
    "design_patterns": {"type": "string"},
    "strengths": {"type": "string"},
    "improvements": {"type": "string"},
    "relationships": {"type": "string"},
    "open_questions": {"type": "string"}
  },
  "required": ["tagline", "type", "status", "tech", "entry_point", "what_it_does", "architecture", "key_files"]
}'

echo "Running Claude analysis (haiku model)..."

# Temp file for Claude output
TEMP_FILE=$(mktemp)
trap "rm -f $TEMP_FILE" EXIT

# Run Claude with read-only tools
if ! claude -p \
    --tools "Read,Glob,Grep" \
    --output-format json \
    --json-schema "$SCHEMA" \
    --add-dir "$PROJECT_PATH" \
    --permission-mode bypassPermissions \
    --model haiku \
    "$PROMPT" > "$TEMP_FILE" 2>&1; then
    echo "Error: Claude CLI returned non-zero exit code"
    echo "Output:"
    head -20 "$TEMP_FILE"
    exit 1
fi

ANALYSIS_JSON=$(cat "$TEMP_FILE")

if [[ -z "$ANALYSIS_JSON" ]]; then
    echo "Error: Claude returned empty response"
    exit 1
fi

# Check if response is valid JSON
if ! echo "$ANALYSIS_JSON" | jq -e '.' > /dev/null 2>&1; then
    echo "Error: Claude returned invalid JSON"
    echo "First 500 chars:"
    head -c 500 "$TEMP_FILE"
    exit 1
fi

# Extract structured_output from the response array
RESULT=$(echo "$ANALYSIS_JSON" | jq '.[-1].structured_output // empty' 2>/dev/null)

if [[ -z "$RESULT" ]] || [[ "$RESULT" == "null" ]]; then
    # Try to find it elsewhere in the array
    RESULT=$(echo "$ANALYSIS_JSON" | jq 'map(select(.structured_output != null)) | .[0].structured_output // empty' 2>/dev/null)
fi

if [[ -z "$RESULT" ]] || [[ "$RESULT" == "null" ]]; then
    echo "Error: No structured_output in Claude response"
    # Show what we got
    echo "Response type: $(echo "$ANALYSIS_JSON" | jq -r 'type')"
    echo "Keys in last element: $(echo "$ANALYSIS_JSON" | jq -r '.[-1] | keys | join(", ")' 2>/dev/null || echo 'N/A')"
    COST=$(echo "$ANALYSIS_JSON" | jq -r '.[-1].total_cost_usd // "unknown"' 2>/dev/null)
    echo "Cost: $COST USD"
    exit 1
fi

echo "Generating analysis file..."

# Extract fields
TAGLINE=$(echo "$RESULT" | jq -r '.tagline // "-"')
TYPE=$(echo "$RESULT" | jq -r '.type // "Unknown"')
STATUS=$(echo "$RESULT" | jq -r '.status // "Unknown"')
TECH=$(echo "$RESULT" | jq -r '.tech // "unknown"')
ENTRY=$(echo "$RESULT" | jq -r '.entry_point // "-"')
WHAT_IT_DOES=$(echo "$RESULT" | jq -r '.what_it_does // ""')
ARCHITECTURE=$(echo "$RESULT" | jq -r '.architecture // ""')
MODULE_BREAKDOWN=$(echo "$RESULT" | jq -r '.module_breakdown // ""')
DESIGN_PATTERNS=$(echo "$RESULT" | jq -r '.design_patterns // ""')
STRENGTHS=$(echo "$RESULT" | jq -r '.strengths // ""')
IMPROVEMENTS=$(echo "$RESULT" | jq -r '.improvements // ""')
RELATIONSHIPS=$(echo "$RESULT" | jq -r '.relationships // ""')
OPEN_QUESTIONS=$(echo "$RESULT" | jq -r '.open_questions // ""')

# Build key files table
KEY_FILES_TABLE=""
while IFS= read -r row; do
    if [[ -n "$row" ]]; then
        FILE=$(echo "$row" | jq -r '.file')
        PURPOSE=$(echo "$row" | jq -r '.purpose')
        KEY_FILES_TABLE="${KEY_FILES_TABLE}| ${FILE} | ${PURPOSE} |
"
    fi
done < <(echo "$RESULT" | jq -c '.key_files[]?' 2>/dev/null)

# Generate the markdown file directly
cat > "$OUTPUT_FILE" << ENDOFFILE
# ${PROJECT_NAME}

> ${TAGLINE}

## Summary

**Type**: ${TYPE}
**Status**: ${STATUS}
**Tech**: ${TECH}
**Entry Point**: ${ENTRY}
**Last Analyzed**: ${TODAY}

### What It Does

${WHAT_IT_DOES}

### Architecture Overview

${ARCHITECTURE}

### Key Files

| File | Purpose |
|------|---------|
${KEY_FILES_TABLE}
ENDOFFILE

# Add deep dive section if not summary-only
if [[ "$SUMMARY_ONLY" != "--summary-only" ]]; then
    cat >> "$OUTPUT_FILE" << ENDOFFILE

---

## Deep Dive

### Module Breakdown

${MODULE_BREAKDOWN}

### Design Patterns

${DESIGN_PATTERNS}

### Code Quality Notes

**Strengths**:
${STRENGTHS}

**Areas for Improvement**:
${IMPROVEMENTS}

### Relationship Map

${RELATIONSHIPS}

### Open Questions

${OPEN_QUESTIONS}
ENDOFFILE
fi

COST=$(echo "$ANALYSIS_JSON" | jq -r '.[-1].total_cost_usd // "unknown"')

# Update _meta.json
META_FILE="$OUTPUT_DIR/_meta.json"
ANALYZED_AT=$(date -u +"%Y-%m-%dT%H:%M:%S")
ANALYZED_COMMIT=$(git -C "$PROJECT_PATH" rev-parse --short HEAD 2>/dev/null || echo "unknown")

if [[ -f "$META_FILE" ]]; then
    # Update existing meta file
    TMP_META=$(mktemp)
    jq --arg name "$PROJECT_NAME" \
       --arg at "$ANALYZED_AT" \
       --arg commit "$ANALYZED_COMMIT" \
       '.projects[$name] = {analyzed_at: $at, analyzed_commit: $commit}' \
       "$META_FILE" > "$TMP_META" && mv "$TMP_META" "$META_FILE"
else
    # Create new meta file
    echo "{\"version\":1,\"projects\":{\"$PROJECT_NAME\":{\"analyzed_at\":\"$ANALYZED_AT\",\"analyzed_commit\":\"$ANALYZED_COMMIT\"}}}" | jq '.' > "$META_FILE"
fi

echo ""
echo "Done! Analysis file: $OUTPUT_FILE"
echo "Cost: \$${COST}"
