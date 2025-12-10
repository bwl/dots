#!/usr/bin/env bash
# Generate markdown dashboard of all ideas
# Usage: _scripts/mq/dashboard.sh > DASHBOARD.md

cd "$(git rev-parse --show-toplevel)"

echo "# Ideas Dashboard"
echo
echo "Generated: $(date +%Y-%m-%d)"
echo
echo "| Idea | Status | Open Questions | Sessions |"
echo "|------|--------|----------------|----------|"

for f in */README.md; do
    folder=$(dirname "$f")
    [[ "$folder" == _* ]] && continue

    # Get status from README
    status=$(mq '.' "$f" 2>/dev/null | grep -m1 "Status:" | sed 's/.*Status:[[:space:]]*//' | sed 's/\*\*//g' | tr -d '*' | xargs || true)
    status="${status:-unknown}"

    # Count open questions (sum all grep counts)
    questions=$(mq '.list' "$f" 2>/dev/null | grep -c '\[ \]' 2>/dev/null | awk '{s+=$1} END {print s+0}')

    # Get sessions from tracker (using awk for proper CSV parsing)
    sessions=$(awk -F',' -v folder="$folder" '
        $1 == folder {
            # Last field is sessions
            print $NF
        }
    ' _tracker.csv 2>/dev/null || echo "-")

    echo "| $folder | $status | $questions | $sessions |"
done

echo
echo "---"
echo
echo "## By Status"
echo

# Count by status using temp file for proper handling
tmpfile=$(mktemp)
trap "rm -f $tmpfile" EXIT

for f in */README.md; do
    folder=$(dirname "$f")
    [[ "$folder" == _* ]] && continue
    status=$(mq '.' "$f" 2>/dev/null | grep -m1 "Status:" | sed 's/.*Status:[[:space:]]*//' | sed 's/\*\*//g' | tr -d '*' | xargs || true)
    status="${status:-unknown}"
    echo "$status" >> "$tmpfile"
done

sort "$tmpfile" | uniq -c | sort -rn | while read count status; do
    echo "- **$status**: $count"
done
