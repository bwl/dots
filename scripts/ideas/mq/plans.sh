#!/usr/bin/env bash
# List all Claude plans with titles and modification dates
# Usage: _scripts/mq/plans.sh

PLANS_DIR="$HOME/.claude/plans"

if [[ ! -d "$PLANS_DIR" ]]; then
    echo "No plans directory found at $PLANS_DIR" >&2
    exit 1
fi

# Collect: date|filename|title
for f in "$PLANS_DIR"/*.md; do
    [[ -f "$f" ]] || continue

    mod=$(stat -f "%Sm" -t "%Y-%m-%d" "$f")
    name=$(basename "$f" .md)
    title=$(mq '.h1' "$f" 2>/dev/null | head -1 | sed 's/^# //')
    title="${title:-(no title)}"

    printf '%s|%s|%s\n' "$mod" "$name" "$title"
done | sort -r | while IFS='|' read -r date name title; do
    printf '%s | %s | %s\n' "$name" "$title" "$date"
done
