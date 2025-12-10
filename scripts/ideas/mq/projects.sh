#!/usr/bin/env bash
# List all projects from the cached inventory
# Usage: projects.sh [--refresh] [--json] [--recent]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
INVENTORY="$REPO_ROOT/_data/project-inventory.json"

# Parse args
REFRESH=false
JSON=false
RECENT=false

for arg in "$@"; do
    case "$arg" in
        --refresh) REFRESH=true ;;
        --json) JSON=true ;;
        --recent) RECENT=true ;;
        -h|--help)
            echo "Usage: projects.sh [--refresh] [--json] [--recent]"
            echo "  --refresh  Regenerate cache before listing"
            echo "  --json     Output raw JSON"
            echo "  --recent   Sort by last commit (default: alphabetical)"
            exit 0
            ;;
    esac
done

# Refresh if requested or cache doesn't exist
if [[ "$REFRESH" == "true" || ! -f "$INVENTORY" ]]; then
    "$SCRIPT_DIR/projects-scan.sh" >&2
fi

if [[ ! -f "$INVENTORY" ]]; then
    echo "Error: No inventory found. Run with --refresh" >&2
    exit 1
fi

if [[ "$JSON" == "true" ]]; then
    cat "$INVENTORY"
    exit 0
fi

# Format output with jq
if [[ "$RECENT" == "true" ]]; then
    # Sort by last_commit descending, then name
    jq -r '.projects | sort_by(.last_commit) | reverse | .[] |
        "\(.name)|\(.category)|\(.last_commit)|\(.source)|\(.description)"' "$INVENTORY"
else
    # Sort alphabetically by name
    jq -r '.projects | sort_by(.name) | .[] |
        "\(.name)|\(.category)|\(.last_commit)|\(.source)|\(.description)"' "$INVENTORY"
fi | while IFS='|' read -r name cat commit src desc; do
    # Truncate description to 40 chars
    desc="${desc:0:40}"
    printf '%-20s | %-10s | %-10s | %-9s | %s\n' "$name" "$cat" "$commit" "$src" "$desc"
done
