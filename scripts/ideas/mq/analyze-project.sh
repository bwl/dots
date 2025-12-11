#!/bin/bash
# analyze-project.sh - Generate analysis file for a project
# Usage: analyze-project.sh <project-path> [--summary-only]
#
# This script gathers metadata about a project and generates a markdown
# analysis file. For deep analysis content, it outputs a scaffold that
# can be filled in by Claude or the user.

set -uo pipefail

PROJECT_PATH="$1"
SUMMARY_ONLY="${2:-}"

if [[ -z "$PROJECT_PATH" ]]; then
    echo "Usage: analyze-project.sh <project-path> [--summary-only]"
    exit 1
fi

if [[ ! -d "$PROJECT_PATH" ]]; then
    echo "Error: Directory not found: $PROJECT_PATH"
    exit 1
fi

PROJECT_NAME=$(basename "$PROJECT_PATH")
OUTPUT_DIR="$HOME/Developer/ideas/_data/analysis"
OUTPUT_FILE="$OUTPUT_DIR/$PROJECT_NAME.md"
TODAY=$(date +%Y-%m-%d)

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

# ============ Helper Functions ============

detect_tech() {
    local dir="$1"
    local tech=""

    # Rust
    if [[ -f "$dir/Cargo.toml" ]]; then
        tech="rust"
        if grep -q 'bevy' "$dir/Cargo.toml" 2>/dev/null; then
            tech="rust, bevy"
        elif grep -q 'ratatui' "$dir/Cargo.toml" 2>/dev/null; then
            tech="rust, ratatui"
        elif grep -q 'clap' "$dir/Cargo.toml" 2>/dev/null; then
            tech="rust, cli"
        fi
    # Node/TypeScript
    elif [[ -f "$dir/package.json" ]]; then
        if [[ -f "$dir/tsconfig.json" ]]; then
            tech="typescript"
        else
            tech="javascript"
        fi
        if grep -q '"next"' "$dir/package.json" 2>/dev/null; then
            tech="$tech, next.js"
        elif grep -q '"react"' "$dir/package.json" 2>/dev/null; then
            tech="$tech, react"
        fi
    # Python
    elif [[ -f "$dir/pyproject.toml" ]] || [[ -f "$dir/setup.py" ]] || [[ -f "$dir/requirements.txt" ]]; then
        tech="python"
    # Go
    elif [[ -f "$dir/go.mod" ]]; then
        tech="go"
    # Make/C
    elif [[ -f "$dir/Makefile" ]]; then
        tech="make/c"
    else
        tech="unknown"
    fi

    echo "$tech"
}

find_entry_point() {
    local dir="$1"

    if [[ -f "$dir/src/main.rs" ]]; then
        echo "src/main.rs"
    elif [[ -f "$dir/src/lib.rs" ]]; then
        echo "src/lib.rs"
    elif [[ -f "$dir/src/index.ts" ]]; then
        echo "src/index.ts"
    elif [[ -f "$dir/src/index.js" ]]; then
        echo "src/index.js"
    elif [[ -f "$dir/index.ts" ]]; then
        echo "index.ts"
    elif [[ -f "$dir/index.js" ]]; then
        echo "index.js"
    elif [[ -f "$dir/main.py" ]]; then
        echo "main.py"
    elif [[ -f "$dir/app.py" ]]; then
        echo "app.py"
    elif [[ -f "$dir/main.go" ]]; then
        echo "main.go"
    else
        echo "-"
    fi
}

get_description() {
    local dir="$1"
    local desc=""

    # Try README
    if [[ -f "$dir/README.md" ]]; then
        # Get first non-empty, non-header line
        desc=$(grep -v '^#' "$dir/README.md" | grep -v '^$' | head -1 | cut -c1-100)
    fi

    # Try package.json description
    if [[ -z "$desc" ]] && [[ -f "$dir/package.json" ]]; then
        desc=$(grep '"description"' "$dir/package.json" 2>/dev/null | head -1 | sed 's/.*"description"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
    fi

    # Try Cargo.toml description
    if [[ -z "$desc" ]] && [[ -f "$dir/Cargo.toml" ]]; then
        desc=$(grep '^description' "$dir/Cargo.toml" 2>/dev/null | head -1 | sed 's/.*=[[:space:]]*"\([^"]*\)".*/\1/')
    fi

    echo "${desc:--}"
}

count_source_files() {
    local dir="$1"
    local count=0

    # Count common source files
    count=$(find "$dir" -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.py" -o -name "*.go" \) 2>/dev/null | grep -v node_modules | grep -v target | grep -v .git | wc -l | tr -d ' ')

    echo "$count"
}

get_git_info() {
    local dir="$1"

    if [[ -d "$dir/.git" ]]; then
        local commits=$(git -C "$dir" rev-list --count HEAD 2>/dev/null || echo "0")
        local last_commit=$(git -C "$dir" log -1 --format="%cd" --date=short 2>/dev/null || echo "-")
        local branch=$(git -C "$dir" branch --show-current 2>/dev/null || echo "-")
        echo "$commits|$last_commit|$branch"
    else
        echo "0|-|-"
    fi
}

list_key_files() {
    local dir="$1"

    # List important files that exist
    local files=""

    for f in "src/main.rs" "src/lib.rs" "src/index.ts" "src/app.ts" "main.py" "app.py" \
             "Cargo.toml" "package.json" "pyproject.toml" "go.mod" \
             "README.md" "CLAUDE.md" ".env.example"; do
        if [[ -f "$dir/$f" ]]; then
            files="$files$f\n"
        fi
    done

    # List src/ subdirectories (modules)
    if [[ -d "$dir/src" ]]; then
        for d in "$dir/src"/*/; do
            if [[ -d "$d" ]]; then
                local subdir=$(basename "$d")
                files="$files src/$subdir/\n"
            fi
        done
    fi

    # List crates/ if it exists (Rust workspace)
    if [[ -d "$dir/crates" ]]; then
        local crate_count=$(ls -1 "$dir/crates" 2>/dev/null | wc -l | tr -d ' ')
        files="$files crates/ ($crate_count crates)\n"
    fi

    echo -e "$files"
}

# ============ Gather Data ============

TECH=$(detect_tech "$PROJECT_PATH")
ENTRY=$(find_entry_point "$PROJECT_PATH")
DESC=$(get_description "$PROJECT_PATH")
SRC_COUNT=$(count_source_files "$PROJECT_PATH")

IFS='|' read -r GIT_COMMITS GIT_LAST GIT_BRANCH <<< "$(get_git_info "$PROJECT_PATH")"

# Determine project type
PROJECT_TYPE="Project"
if [[ "$TECH" == *"bevy"* ]]; then
    PROJECT_TYPE="Bevy game"
elif [[ "$TECH" == *"ratatui"* ]]; then
    PROJECT_TYPE="TUI application"
elif [[ "$TECH" == *"cli"* ]]; then
    PROJECT_TYPE="CLI tool"
elif [[ "$TECH" == *"next"* ]] || [[ "$TECH" == *"react"* ]]; then
    PROJECT_TYPE="Web application"
elif [[ "$TECH" == "rust" ]]; then
    PROJECT_TYPE="Rust project"
elif [[ "$TECH" == "typescript" ]] || [[ "$TECH" == "javascript" ]]; then
    PROJECT_TYPE="Node.js project"
elif [[ "$TECH" == "python" ]]; then
    PROJECT_TYPE="Python project"
fi

# Determine activity status
if [[ "$GIT_COMMITS" -gt 100 ]]; then
    STATUS="Active development"
elif [[ "$GIT_COMMITS" -gt 20 ]]; then
    STATUS="Active"
elif [[ "$GIT_COMMITS" -gt 0 ]]; then
    STATUS="Early development"
else
    STATUS="Unknown"
fi

# ============ Generate Output ============

cat > "$OUTPUT_FILE" << EOF
# $PROJECT_NAME

> $DESC

## Summary

**Type**: $PROJECT_TYPE
**Status**: $STATUS
**Tech**: $TECH
**Entry Point**: $ENTRY
**Last Analyzed**: $TODAY

### What It Does

<!-- TODO: 2-3 paragraphs explaining purpose, key features, current state -->

$DESC

### Architecture Overview

- **Source files**: $SRC_COUNT files
- **Git commits**: $GIT_COMMITS (last: $GIT_LAST)
- **Branch**: $GIT_BRANCH

<!-- TODO: Main modules and their responsibilities -->

### Key Files

| File | Purpose |
|------|---------|
EOF

# Add key files to table
while IFS= read -r file; do
    if [[ -n "$file" ]]; then
        echo "| $file | <!-- TODO --> |" >> "$OUTPUT_FILE"
    fi
done <<< "$(list_key_files "$PROJECT_PATH")"

# Add deep dive section if not summary-only
if [[ "$SUMMARY_ONLY" != "--summary-only" ]]; then
    cat >> "$OUTPUT_FILE" << 'EOF'

---

## Deep Dive

### Module Breakdown

<!-- TODO: Detailed analysis of each major module/crate -->

### Design Patterns

<!-- TODO: Notable patterns used in this codebase -->

### Code Quality Notes

**Strengths**:
<!-- TODO -->

**Areas for Improvement**:
<!-- TODO -->

### Relationship Map

<!-- TODO: How this project relates to other projects in the portfolio -->

### Open Questions

<!-- TODO: Things that need investigation or decisions -->
EOF
fi

echo "Generated: $OUTPUT_FILE"
