#!/usr/bin/env bash
# Search Claude plans by title and content
# Usage: _scripts/mq/plans-search.sh <term>

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <search-term>" >&2
    exit 1
fi

PLANS_DIR="$HOME/.claude/plans"
term="$1"

if [[ ! -d "$PLANS_DIR" ]]; then
    echo "No plans directory found at $PLANS_DIR" >&2
    exit 1
fi

count=0
for f in "$PLANS_DIR"/*.md; do
    [[ -f "$f" ]] || continue

    if grep -qiE "$term" "$f" 2>/dev/null; then
        name=$(basename "$f" .md)
        title=$(mq '.h1' "$f" 2>/dev/null | head -1 | sed 's/^# //')
        title="${title:-(no title)}"

        echo "=== $name ==="
        echo "Title: $title"
        echo "---"
        grep -iE --color=never -n "$term" "$f" | head -5
        echo
        ((count++))
    fi
done

echo "Found $count plan(s) matching '$term'"
