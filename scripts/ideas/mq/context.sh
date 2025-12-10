#!/usr/bin/env bash
# Generate focused LLM context for a specific idea
# Usage: _scripts/mq/context.sh <idea-folder>

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <idea-folder>" >&2
    exit 1
fi

idea="$1"
readme="$idea/README.md"

if [[ ! -f "$readme" ]]; then
    echo "Error: $readme not found" >&2
    exit 1
fi

# Title
echo "# $(mq '.h1' "$readme" 2>/dev/null | sed 's/^# //')"
echo

# Status line if present
status_line=$(mq '.' "$readme" 2>/dev/null | grep -m1 "Status:" || true)
if [[ -n "$status_line" ]]; then
    echo "$status_line"
    echo
fi

# First paragraph (intro/description)
mq '.' "$readme" 2>/dev/null | awk '
    /^#/ { next }
    /^\*\*Status/ { next }
    /^[[:space:]]*$/ { if (started) exit; next }
    { started=1; print }
'
echo

# Key bullet points (first 15 lines of lists)
echo "## Key Points"
mq '.list' "$readme" 2>/dev/null | head -15
echo

# Open questions
questions=$(mq '.list' "$readme" 2>/dev/null | grep '\[ \]' || true)
if [[ -n "$questions" ]]; then
    echo "## Open Questions"
    echo "$questions"
    echo
fi

# List other files in the folder
other_files=$(ls -1 "$idea" 2>/dev/null | grep -v '^README.md$' || true)
if [[ -n "$other_files" ]]; then
    echo "## Other Files"
    echo "$other_files" | sed 's/^/- /'
fi
