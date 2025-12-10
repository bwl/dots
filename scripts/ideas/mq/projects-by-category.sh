#!/usr/bin/env bash
# Group projects by category
# Usage: projects-by-category.sh [category]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
INVENTORY="$REPO_ROOT/_data/project-inventory.json"

if [[ ! -f "$INVENTORY" ]]; then
    echo "No inventory found. Running scan..." >&2
    "$SCRIPT_DIR/projects-scan.sh" >&2
fi

FILTER="${1:-}"

if [[ -n "$FILTER" ]]; then
    # Show single category
    echo "=== $FILTER ==="
    jq -r --arg cat "$FILTER" '
        .projects |
        map(select(.category == $cat)) |
        sort_by(.last_commit) | reverse |
        .[] |
        "\(.name)|\(.last_commit)|\(.source)|\(.description)"
    ' "$INVENTORY" | while IFS='|' read -r name commit src desc; do
        printf '%-20s | %-10s | %-9s | %s\n' "$name" "$commit" "$src" "${desc:0:35}"
    done
else
    # Show all categories grouped
    categories=$(jq -r '[.projects[].category] | unique | .[]' "$INVENTORY" | sort)

    for cat in $categories; do
        count=$(jq -r --arg cat "$cat" '[.projects[] | select(.category == $cat)] | length' "$INVENTORY")
        echo "=== $cat ($count) ==="

        jq -r --arg cat "$cat" '
            .projects |
            map(select(.category == $cat)) |
            sort_by(.last_commit) | reverse |
            .[] |
            "\(.name)|\(.last_commit)|\(.source)|\(.description)"
        ' "$INVENTORY" | while IFS='|' read -r name commit src desc; do
            printf '  %-20s | %-10s | %-9s | %s\n' "$name" "$commit" "$src" "${desc:0:30}"
        done
        echo
    done

    # Summary
    total=$(jq '.projects | length' "$INVENTORY")
    echo "=== Summary ==="
    echo "Total: $total projects"
    jq -r '
        [.projects[].category] |
        group_by(.) |
        map({category: .[0], count: length}) |
        sort_by(.count) | reverse |
        .[] |
        "  \(.category): \(.count)"
    ' "$INVENTORY"
fi
