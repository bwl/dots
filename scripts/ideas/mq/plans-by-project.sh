#!/usr/bin/env bash
# Group Claude plans by project keywords
# Usage: _scripts/mq/plans-by-project.sh

PLANS_DIR="$HOME/.claude/plans"

if [[ ! -d "$PLANS_DIR" ]]; then
    echo "No plans directory found at $PLANS_DIR" >&2
    exit 1
fi

# Project patterns: "regex:label"
PROJECTS=(
    "crt|crt-runner:crt"
    "terminal-rogue|rustrogue|ratband|harvester|toftband:roguelike"
    "forest|forest-lsp:forest"
    "pulper|novel:pulper"
    "kingdom|workflow:kingdom"
    "versiontracker:versiontracker"
    "ideas-tui:ideas"
)

tmpfile=$(mktemp)
trap "rm -f $tmpfile" EXIT

# Classify each plan
for f in "$PLANS_DIR"/*.md; do
    [[ -f "$f" ]] || continue

    name=$(basename "$f" .md)
    title=$(mq '.h1' "$f" 2>/dev/null | head -1 | sed 's/^# //')
    title="${title:-(no title)}"

    matched=""
    for proj in "${PROJECTS[@]}"; do
        pattern="${proj%%:*}"
        label="${proj##*:}"
        if grep -qiE "$pattern" "$f" 2>/dev/null; then
            matched="$label"
            break
        fi
    done
    matched="${matched:-misc}"

    printf '%s\t%s\t%s\n' "$matched" "$name" "$title" >> "$tmpfile"
done

# Print grouped
cut -f1 "$tmpfile" | sort | uniq -c | sort -rn | while read -r count label; do
    echo "=== $label ($count) ==="
    awk -F'\t' -v l="$label" '$1 == l { printf "%s | %s\n", $2, $3 }' "$tmpfile" | sort
    echo
done
