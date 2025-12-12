#!/bin/bash
# generate-summary.sh - Use Claude CLI to generate a 1-line project summary
# Usage: generate-summary.sh <project-name>
#
# This script:
# 1. Reads analysis file (if exists) or README
# 2. Uses Claude haiku to generate a concise 1-line summary
# 3. Stores it in _meta.json under "summaries"

set -uo pipefail

PROJECT_NAME="$1"

if [[ -z "$PROJECT_NAME" ]]; then
    echo "Usage: generate-summary.sh <project-name>"
    exit 1
fi

IDEAS_REPO="$HOME/Developer/ideas"
ANALYSIS_FILE="$IDEAS_REPO/_data/analysis/$PROJECT_NAME.md"
META_FILE="$IDEAS_REPO/_data/analysis/_meta.json"

# Find project path from inventory
PROJECT_PATH=$(jq -r ".projects[] | select(.name == \"$PROJECT_NAME\") | .path" "$IDEAS_REPO/_data/project-inventory.json" 2>/dev/null)

if [[ -z "$PROJECT_PATH" ]] || [[ "$PROJECT_PATH" == "null" ]]; then
    echo "Error: Project not found in inventory: $PROJECT_NAME"
    exit 1
fi

# Trim trailing slash
PROJECT_PATH="${PROJECT_PATH%/}"

if [[ ! -d "$PROJECT_PATH" ]]; then
    echo "Error: Project directory not found: $PROJECT_PATH"
    exit 1
fi

echo "=== Generating Summary: $PROJECT_NAME ==="

# Build context from analysis file or README
CONTEXT=""
if [[ -f "$ANALYSIS_FILE" ]]; then
    echo "Using analysis file for context..."
    CONTEXT=$(head -100 "$ANALYSIS_FILE")
elif [[ -f "$PROJECT_PATH/README.md" ]]; then
    echo "Using README for context..."
    CONTEXT=$(head -100 "$PROJECT_PATH/README.md")
else
    echo "Error: No analysis file or README found"
    exit 1
fi

# Build the prompt
PROMPT="Based on this project description, generate a single-sentence summary (max 100 characters) that describes what the project does. Be specific and concrete. Do not start with 'A' or 'This'. Focus on the core functionality.

Project: $PROJECT_NAME

Context:
$CONTEXT

Output only the summary sentence, nothing else."

echo "Running Claude haiku..."

# Temp file for Claude output
TEMP_FILE=$(mktemp)
trap "rm -f $TEMP_FILE" EXIT

# Run Claude without json-schema (simpler output format)
if ! claude -p \
    --output-format json \
    --permission-mode bypassPermissions \
    --model haiku \
    "$PROMPT" > "$TEMP_FILE" 2>&1; then
    echo "Error: Claude CLI returned non-zero exit code"
    cat "$TEMP_FILE"
    exit 1
fi

RESPONSE=$(cat "$TEMP_FILE")

# Extract result from the response (it's in .result field of the last element)
SUMMARY=$(echo "$RESPONSE" | jq -r '.[-1].result // empty' 2>/dev/null)

# Clean up - remove quotes and newlines
SUMMARY=$(echo "$SUMMARY" | tr -d '\n' | sed 's/^"//' | sed 's/"$//' | cut -c1-120)

if [[ -z "$SUMMARY" ]] || [[ "$SUMMARY" == "null" ]]; then
    echo "Error: Could not extract summary from Claude response"
    echo "Response: $RESPONSE"
    exit 1
fi

echo "Summary: $SUMMARY"

# Store in _meta.json
if [[ ! -f "$META_FILE" ]]; then
    echo '{"version":1,"projects":{},"summaries":{}}' > "$META_FILE"
fi

# Add/update the summary
TMP_META=$(mktemp)
jq --arg name "$PROJECT_NAME" \
   --arg summary "$SUMMARY" \
   '.summaries[$name] = $summary' \
   "$META_FILE" > "$TMP_META" && mv "$TMP_META" "$META_FILE"

COST=$(echo "$RESPONSE" | jq -r '.[-1].total_cost_usd // "unknown"')

echo ""
echo "Done! Summary stored in _meta.json"
echo "Cost: \$${COST}"
