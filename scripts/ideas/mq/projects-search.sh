#!/usr/bin/env bash
# Search projects for triage - find if an idea already exists
# Usage: projects-search.sh <term> [--deep]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
INVENTORY="$REPO_ROOT/_data/project-inventory.json"

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <search-term> [--deep]"
    echo "  --deep  Also search README files (slower)"
    exit 1
fi

TERM="$1"
DEEP=false
[[ "${2:-}" == "--deep" ]] && DEEP=true

if [[ ! -f "$INVENTORY" ]]; then
    echo "No inventory found. Running scan..." >&2
    "$SCRIPT_DIR/projects-scan.sh" >&2
fi

# Search JSON fields (name, description, category, tech)
echo "=== Searching inventory for '$TERM' ==="
echo

matches=$(jq -r --arg term "$TERM" '
    .projects[] |
    select(
        (.name | test($term; "i")) or
        (.description | test($term; "i")) or
        (.category | test($term; "i")) or
        (.tech | test($term; "i"))
    ) |
    "\(.name)|\(.category)|\(.source)|\(.tech)|\(.description)|\(.path)"
' "$INVENTORY" 2>/dev/null)

if [[ -z "$matches" ]]; then
    echo "No matches in inventory."
else
    echo "$matches" | while IFS='|' read -r name cat src tech desc path; do
        echo "--- $name ---"
        echo "Category: $cat | Source: $src | Tech: $tech"
        echo "Description: ${desc:0:70}"
        echo "Path: $path"
        echo
    done
fi

# Deep search: look in README files
if [[ "$DEEP" == "true" ]]; then
    echo "=== Deep search (README files) ==="
    echo

    readme_matches=0
    jq -r '.projects[] | "\(.name)|\(.path)"' "$INVENTORY" | while IFS='|' read -r name path; do
        readme="$path/README.md"
        if [[ -f "$readme" ]]; then
            if grep -qiE "$TERM" "$readme" 2>/dev/null; then
                echo "--- $name ---"
                grep -iE --color=never -n "$TERM" "$readme" 2>/dev/null | head -3
                echo
                ((readme_matches++))
            fi
        fi
    done

    if [[ $readme_matches -eq 0 ]]; then
        echo "No additional matches in README files."
    fi
fi
