#!/usr/bin/env bash
# Scan all project directories and generate inventory JSON
# Usage: projects-scan.sh

set -uo pipefail

IDEAS_REPO="$HOME/Developer/ideas"
OUTPUT="$IDEAS_REPO/_data/project-inventory.json"

# Directories to scan
# Note: ideas folders are tracked separately via _tracker.csv, not here
SOURCES=(
    "$HOME/Developer:Developer"
)

# Category patterns: "regex:category"
CATEGORIES=(
    # Roguelikes (12+)
    "rustrogue|terminal-rogue|TOFT|ratband|harvester|inkdom|toftband|bevyRogue|rogueGPT|rlThin|angband|BrogueCE|village-watch|roguelike-engine:roguelike"

    # Writing tools (rename from novels)
    "pulper|novels|novels-gpt|novel_thoughts|steam-novel|write-kit|teller|prose-linter|scriveners-lens|ghostwriters-guild|plot-threads|lorekeeper|chronicle:writing"

    # AI tools
    "kingdom|clevercli|craft|kingclaude|ettio:ai_tools"

    # MCP ecosystem
    "mcp-mpc|tauri-plugin-mcp|tldr-agent-spec:mcp"

    # Knowledge management
    "forest|forest-lsp|forest-cloud|camper|loom|room:knowledge"

    # Simulation
    "fantasyWorld|npcSim|world|farm|life:simulation"

    # Worldbuilding / procgen
    "seedworld|seedworld-cli|worldforge|procgen-api|oldyork|roguelike-worldforge:worldbuilding"

    # CRT terminal renderer
    "crt-runner|crt-runner-types|cartridge:crt"

    # TUI apps (visual terminal interfaces)
    "brew-info-project|crush|fli|gh-stars|oliviaOS|terminal-productivity:tui"

    # CLI tools (command-line utilities)
    "cliffy|marper|graphrite|setup:cli"

    # Minecraft server/plugins
    "minecraft|mcMMO|CoreProtect|juke:minecraft"

    # Web projects
    "benjaminlundquist|ettio-shopify|site-ettio|wny|fiscalsim|fiscal-clean|fiscalsim-serverless:web"

    # Tarot
    "tarot|tarot-mobile:tarot"

    # Other specific
    "versiontracker|versiontracker-tracer-bullet:versiontracker"
    "ideas-tui|ideas|repo-enhancements|project-inventory:ideas"
    "entititty:ecs"

    # Personal sites/docs
    "bwl|bwl-profile|KristenBook|harvestBook1:personal"
)

# Get category for a project name
get_category() {
    local name="$1"
    for cat in "${CATEGORIES[@]}"; do
        local pattern="${cat%%:*}"
        local label="${cat##*:}"
        if echo "$name" | grep -qiE "^($pattern)$"; then
            echo "$label"
            return
        fi
    done
    echo ""
}

# Detect tech stack from project files
detect_tech() {
    local dir="$1"
    local tech=""

    [[ -f "$dir/Cargo.toml" ]] && tech="rust"
    [[ -f "$dir/package.json" ]] && tech="${tech:+$tech,}js"
    [[ -f "$dir/pyproject.toml" || -f "$dir/setup.py" ]] && tech="${tech:+$tech,}python"
    [[ -f "$dir/go.mod" ]] && tech="${tech:+$tech,}go"
    [[ -f "$dir/Makefile" && -z "$tech" ]] && tech="make"

    # Check for specific frameworks in Cargo.toml
    if [[ -f "$dir/Cargo.toml" ]]; then
        grep -q 'bevy' "$dir/Cargo.toml" 2>/dev/null && tech="${tech:+$tech,}bevy"
        grep -q 'ratatui' "$dir/Cargo.toml" 2>/dev/null && tech="${tech:+$tech,}tui"
        grep -q 'clap' "$dir/Cargo.toml" 2>/dev/null && tech="${tech:+$tech,}cli"
    fi

    echo "${tech:-unknown}"
}

# Get description from README or package file
get_description() {
    local dir="$1"
    local desc=""

    # Try README.md first line after title
    if [[ -f "$dir/README.md" ]]; then
        desc=$(sed -n '1,10p' "$dir/README.md" | grep -v '^#' | grep -v '^$' | head -1 | cut -c1-80)
    fi

    # Try Cargo.toml description
    if [[ -z "$desc" && -f "$dir/Cargo.toml" ]]; then
        desc=$(grep '^description' "$dir/Cargo.toml" 2>/dev/null | head -1 | sed 's/description = "\(.*\)"/\1/' | cut -c1-80)
    fi

    # Try package.json description
    if [[ -z "$desc" && -f "$dir/package.json" ]]; then
        desc=$(grep '"description"' "$dir/package.json" 2>/dev/null | head -1 | sed 's/.*"description": "\([^"]*\)".*/\1/' | cut -c1-80)
    fi

    echo "${desc:--}"
}

# Get git info
get_git_info() {
    local dir="$1"
    if [[ -d "$dir/.git" ]]; then
        local last_commit commits
        last_commit=$(git -C "$dir" log -1 --format='%as' 2>/dev/null || echo "-")
        commits=$(git -C "$dir" rev-list --count HEAD 2>/dev/null || echo "0")
        echo "$last_commit|$commits"
    else
        echo "-|0"
    fi
}

# JSON escape
json_escape() {
    echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/	/\\t/g' | tr -d '\n\r'
}

# Main scan
echo "Scanning projects..." >&2

mkdir -p "$(dirname "$OUTPUT")"

# Start JSON
cat > "$OUTPUT" <<EOF
{
  "generated": "$(date -Iseconds)",
  "sources": {},
  "projects": [
EOF

first=true
declare -A source_counts

for source_spec in "${SOURCES[@]}"; do
    dir="${source_spec%%:*}"
    source_name="${source_spec##*:}"

    [[ ! -d "$dir" ]] && continue

    count=0
    for project in "$dir"/*/; do
        [[ ! -d "$project" ]] && continue

        name=$(basename "$project")

        # Skip hidden and underscore-prefixed
        [[ "$name" == .* || "$name" == _* ]] && continue

        # Skip ideas subdirectories (they're not projects)
        [[ "$source_name" == "ideas" && ! -f "$project/README.md" ]] && continue

        ((count++))

        # Get project data
        category=$(get_category "$name")
        tech=$(detect_tech "$project")
        desc=$(get_description "$project")
        git_info=$(get_git_info "$project")
        last_commit="${git_info%%|*}"
        commits="${git_info##*|}"

        # Auto-categorize by tech if no category match
        if [[ -z "$category" ]]; then
            case "$tech" in
                *bevy*) category="bevy" ;;
                *tui*) category="tui" ;;
                *) category="misc" ;;
            esac
        fi

        # Output JSON object
        [[ "$first" != "true" ]] && echo "," >> "$OUTPUT"
        first=false

        cat >> "$OUTPUT" <<EOF
    {
      "name": "$(json_escape "$name")",
      "path": "$(json_escape "$project")",
      "source": "$source_name",
      "category": "$category",
      "tech": "$(json_escape "$tech")",
      "last_commit": "$last_commit",
      "commits": $commits,
      "description": "$(json_escape "$desc")"
    }
EOF
    done

    source_counts[$source_name]=$count
    echo "  $source_name: $count projects" >&2
done

# Close JSON array and add source counts
cat >> "$OUTPUT" <<EOF

  ]
}
EOF

# Patch in source counts (simple sed replacement)
sources_json=""
for src in "${!source_counts[@]}"; do
    [[ -n "$sources_json" ]] && sources_json="$sources_json, "
    sources_json="$sources_json\"$src\": ${source_counts[$src]}"
done
sed -i '' "s/\"sources\": {}/\"sources\": {$sources_json}/" "$OUTPUT"

total=${source_counts[Developer]:-0}
echo "Done. $total projects saved to $OUTPUT" >&2
