#!/usr/bin/env bash
# List all ideas grouped by status
# Usage: _scripts/mq/status.sh

cd "$(git rev-parse --show-toplevel)"

# Collect all folder:status pairs into a temp file (tab-separated)
tmpfile=$(mktemp)
trap "rm -f $tmpfile" EXIT

for f in */README.md; do
    folder=$(dirname "$f")
    [[ "$folder" == _* ]] && continue

    # Extract status and trim whitespace
    raw_status=$(mq '.' "$f" 2>/dev/null | grep -m1 "Status:" | sed 's/.*Status:[[:space:]]*//' | sed 's/\*\*//g' | tr -d '*' || true)
    # Trim leading/trailing whitespace
    status=$(echo "$raw_status" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
    status="${status:-unknown}"
    printf '%s\t%s\n' "$status" "$folder" >> "$tmpfile"
done

# Get unique statuses and print grouped
cut -f1 "$tmpfile" | sort -u | while read -r status; do
    [[ -z "$status" ]] && continue
    echo "=== $status ==="
    awk -F'\t' -v s="$status" '$1 == s { print $2 }' "$tmpfile" | sort
    echo
done
