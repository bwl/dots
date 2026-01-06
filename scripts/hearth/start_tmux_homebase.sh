#!/usr/bin/env bash
set -euo pipefail

# Entrypoint script for starting tuios (Terminal UI Operating System) with AI coding assistants.
# Run from any project directory to create a tuios workspace for that project.
#
# Usage:
#   cd /path/to/my-project
#   hearth                    # Launch tuios in current directory
#   hearth machine            # Show machine report (static snapshot)
#   hearth machine live       # Show live updating machine report
#   hearth machine mock       # Show machine report with mock data
#   hearth machine mock live  # Show live updating mock report
#   hearth --help             # Show tuios help
#
# Note: For tmux-based sessions, use the 'leaf' command instead.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Handle machine report subcommands
if [[ $# -ge 1 && "$1" == "machine" ]]; then
  shift  # remove 'machine' from args

  # Parse flags
  CONTINUOUS=0
  MOCK=0

  for arg in "$@"; do
    case "$arg" in
      live)
        CONTINUOUS=1
        ;;
      mock)
        MOCK=1
        ;;
      *)
        echo "Unknown machine subcommand: $arg" >&2
        echo "Usage: hearth machine [live] [mock]" >&2
        exit 1
        ;;
    esac
  done

  # Set environment and run
  if [[ $CONTINUOUS -eq 1 ]]; then
    export CONTINUOUS_MODE=1
  fi
  if [[ $MOCK -eq 1 ]]; then
    export USE_MOCK_DATA=1
  fi

  exec "${SCRIPT_DIR}/machine_report_dynamic.sh"
fi

if ! command -v tuios >/dev/null 2>&1; then
  echo "tuios is not installed or not discoverable in PATH." >&2
  echo "Build and install tuios from ~/dotfiles/libraries/tuios" >&2
  exit 1
fi

# Pass through any arguments to tuios (e.g., --ssh, --port, etc.)
if [[ $# -gt 0 ]]; then
  exec tuios "$@"
else
  # Launch tuios in the current project directory
  exec tuios
fi
