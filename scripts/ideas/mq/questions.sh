#!/usr/bin/env bash
# Aggregate all open questions (unchecked checkboxes) across ideas
# Usage: _scripts/mq/questions.sh [folder]

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

if [[ $# -eq 1 ]]; then
    # Single folder mode
    folder="$1"
    if [[ -f "$folder/README.md" ]]; then
        questions=$(mq '.list' "$folder/README.md" 2>/dev/null | grep '\[ \]' || true)
        if [[ -n "$questions" ]]; then
            echo "=== $folder ==="
            echo "$questions"
        else
            echo "No open questions in $folder"
        fi
    else
        echo "Error: $folder/README.md not found" >&2
        exit 1
    fi
else
    # All folders mode
    total=0
    for f in */README.md; do
        folder=$(dirname "$f")
        [[ "$folder" == _* ]] && continue

        questions=$(mq '.list' "$f" 2>/dev/null | grep '\[ \]' || true)
        if [[ -n "$questions" ]]; then
            count=$(echo "$questions" | wc -l | xargs)
            total=$((total + count))
            echo "=== $folder ($count) ==="
            echo "$questions"
            echo
        fi
    done
    echo "Total open questions: $total"
fi
