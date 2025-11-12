#!/usr/bin/env bash
# .leaf/launch.sh - Environment bootstrap for Codex and other tools
#
# This wrapper ensures consistent configuration across leaf windows.
# Copy this template to your project's .leaf/ directory and customize as needed.

set -euo pipefail

# Determine project root (one directory up from .leaf/)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Export Codex configuration
export CODEX_CONFIG_PATH="${CODEX_CONFIG_PATH:-$PROJECT_ROOT/.leaf/codex.toml}"

# Optional: Override Codex settings via environment
# export CODEX_LOG_FORMAT="${CODEX_LOG_FORMAT:-compact}"
# export CODEX_OTEL_DEBUG="${CODEX_OTEL_DEBUG:-1}"

# Optional: OTEL endpoint for telemetry
# export OTEL_EXPORTER_OTLP_ENDPOINT="${OTEL_EXPORTER_OTLP_ENDPOINT:-http://127.0.0.1:4317}"

# Execute the provided command with the environment configured
exec "$@"
