#!/usr/bin/env bash
# Initialize _tracker.csv from git history
# Run once to bootstrap, then pre-commit hook maintains it

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TRACKER="$REPO_ROOT/_tracker.csv"

cd "$REPO_ROOT"

# Header
echo 'folder,tags,description,created,modified,sessions' > "$TRACKER"

# Find all root-level directories, excluding _ prefixed ones and .git
for dir in */; do
    folder="${dir%/}"

    # Skip underscore-prefixed directories
    [[ "$folder" == _* ]] && continue

    # Get first commit date for this folder (created)
    created=$(git log --follow --format=%cs --diff-filter=A -- "$folder/" 2>/dev/null | tail -1)
    if [[ -z "$created" ]]; then
        # Fallback: earliest commit touching this folder
        created=$(git log --reverse --format=%cs -- "$folder/" 2>/dev/null | head -1)
    fi
    [[ -z "$created" ]] && created="unknown"

    # Get last commit date (modified)
    modified=$(git log -1 --format=%cs -- "$folder/" 2>/dev/null)
    [[ -z "$modified" ]] && modified="$created"

    # Count commits touching this folder (sessions)
    sessions=$(git log --oneline -- "$folder/" 2>/dev/null | wc -l | tr -d ' ')

    # Tags and description start empty
    echo "$folder,\"\",\"\",$created,$modified,$sessions" >> "$TRACKER"
done

echo "Created $TRACKER with $(tail -n +2 "$TRACKER" | wc -l | tr -d ' ') idea folders"
echo "Edit tags and descriptions manually, then commit."
