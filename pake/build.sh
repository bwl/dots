#!/usr/bin/env bash
# Pake app builder - reads Pakefile, builds desktop apps
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PAKEFILE="$SCRIPT_DIR/Pakefile"
OUTPUT_DIR="$HOME/Applications/Pake"
BUILD_DIR="$SCRIPT_DIR/.build"

# Parse arguments
TARGET_APP=""
while [[ $# -gt 0 ]]; do
  case $1 in
    --app) TARGET_APP="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR" "$BUILD_DIR"

# Extract domain from URL
get_domain() {
  echo "$1" | sed -E 's|https?://([^/]+).*|\1|'
}

# Build a single app
build_app() {
  local url="$1"
  local name="$2"
  shift 2
  local opts=("$@")

  local domain
  domain=$(get_domain "$url")
  local app_dir="$SCRIPT_DIR/apps/$domain"
  local inject_args=()
  local combined_js="$BUILD_DIR/${domain}.js"

  echo "Building: $name ($domain)"

  # Combine harness + site JS
  cat "$SCRIPT_DIR/lib/harness.js" > "$combined_js"
  if [[ -f "$app_dir/inject.js" ]]; then
    echo "" >> "$combined_js"
    cat "$app_dir/inject.js" >> "$combined_js"
  fi
  inject_args+=("$combined_js")

  # Add CSS if exists
  if [[ -f "$app_dir/inject.css" ]] && [[ -s "$app_dir/inject.css" ]]; then
    inject_args+=("$app_dir/inject.css")
  fi

  # Build inject argument
  local inject_str
  inject_str=$(IFS=,; echo "${inject_args[*]}")

  # Run pake
  echo "  pake $url --name $name --inject $inject_str ${opts[*]:-}"
  pake "$url" --name "$name" --inject "$inject_str" "${opts[@]:-}"

  # Move to output directory
  if [[ -f "${name}.app" ]]; then
    mv "${name}.app" "$OUTPUT_DIR/"
    echo "  -> $OUTPUT_DIR/${name}.app"
  elif [[ -d "${name}.app" ]]; then
    rm -rf "$OUTPUT_DIR/${name}.app"
    mv "${name}.app" "$OUTPUT_DIR/"
    echo "  -> $OUTPUT_DIR/${name}.app"
  fi
}

# Parse Pakefile and build
while IFS= read -r line || [[ -n "$line" ]]; do
  # Skip comments and empty lines
  [[ "$line" =~ ^[[:space:]]*# ]] && continue
  [[ -z "${line// }" ]] && continue

  # Parse: pake "URL" "Name" [options]
  if [[ "$line" =~ ^pake[[:space:]]+ ]]; then
    # Extract quoted URL and Name
    url=$(echo "$line" | grep -oE '"https?://[^"]+"' | head -1 | tr -d '"')
    name=$(echo "$line" | grep -oE '"[^"]+"' | sed -n '2p' | tr -d '"')
    # Extract remaining options
    opts=$(echo "$line" | sed -E 's/^pake[[:space:]]+"[^"]+"[[:space:]]+"[^"]+"//')

    # Filter by target app if specified
    if [[ -n "$TARGET_APP" ]] && [[ "$name" != "$TARGET_APP" ]]; then
      continue
    fi

    # shellcheck disable=SC2086
    build_app "$url" "$name" $opts
  fi
done < "$PAKEFILE"

echo "Done!"
