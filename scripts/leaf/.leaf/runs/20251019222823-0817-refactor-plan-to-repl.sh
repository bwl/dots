#!/usr/bin/env bash
set -euo pipefail

STATUS_FILE=/Users/bwl/dotfiles/scripts/leaf/.leaf/history/20251019222823-0817-refactor-plan-to-repl.log
WAIT_KEY=leaf-plan-20251019222823-0817
RUN_LABEL=Refactor\ leaf\ plan\ to\ Use\ Interactive\ Codex\ REPL
RUN_SLUG=refactor-plan-to-repl
RUN_KEY=20251019222823-0817
PROJECT_ROOT=/Users/bwl/dotfiles/scripts/leaf
INSTRUCTIONS_FILE=/Users/bwl/dotfiles/scripts/leaf/.leaf/runs/20251019222823-0817-refactor-plan-to-repl-instructions.txt
SUMMARY_FILE=/Users/bwl/dotfiles/scripts/leaf/.leaf/history/20251019222823-0817-refactor-plan-to-repl-summary.md
EXIT_CODE_FILE=/Users/bwl/dotfiles/scripts/leaf/.leaf/status/20251019222823-0817-refactor-plan-to-repl.exit
TIMEOUT=900
PREVIEW=false
LAUNCH_SCRIPT="$PROJECT_ROOT/.leaf/launch.sh"

mkdir -p "$(dirname "$STATUS_FILE")" "$(dirname "$SUMMARY_FILE")" "$(dirname "$EXIT_CODE_FILE")"

exec > >(stdbuf -oL tee -a "$STATUS_FILE") 2>&1

START_TS=$(date -Iseconds)
echo "[leaf plan] Plan: $RUN_LABEL"
echo "[leaf plan] Run ID: $RUN_KEY"
echo "[leaf plan] Started: $START_TS"

if [[ -n "$INSTRUCTIONS_FILE" && -f "$INSTRUCTIONS_FILE" ]]; then
  echo "[leaf plan] Instructions:"
  sed 's/^/  /' "$INSTRUCTIONS_FILE"
fi

if [[ -z "${CODEX_CONFIG_PATH:-}" && -f "$PROJECT_ROOT/.leaf/codex.toml" ]]; then
  export CODEX_CONFIG_PATH="$PROJECT_ROOT/.leaf/codex.toml"
fi

if [[ ! -x "$LAUNCH_SCRIPT" ]]; then
  echo "[leaf plan] Missing executable launch script at $LAUNCH_SCRIPT" >&2
  exit 1
fi

CMD=()
CMD+=("$LAUNCH_SCRIPT")
CMD+=("codex" "exec")
if [[ -n "${CODEX_CONFIG_PATH:-}" ]]; then
  CMD+=("--config" "$CODEX_CONFIG_PATH")
fi
CMD+=("-C" "$PROJECT_ROOT")
CMD+=("--sandbox" "workspace-write")
CMD+=("-c" "logging.level=debug")
CMD+=("-c" "telemetry.otel.enabled=true")
CMD+=("-c" "telemetry.otel.exporter=debug")
if [[ -n "$INSTRUCTIONS_FILE" && -f "$INSTRUCTIONS_FILE" ]]; then
  CMD+=("-")
fi
if [[ "$PREVIEW" == "true" ]]; then
  echo "[leaf plan] Preview mode enabled (command not executed)."
  printf '[leaf plan] Command:'
  printf ' %q' "${CMD[@]}"
  printf '\n'
  if [[ -n "${TMUX_PANE:-}" ]]; then
    tmux pipe-pane -t "${TMUX_PANE}"
  fi
  tmux wait-for -S "$WAIT_KEY"
  exit 0
fi

exit_code=0
if [[ -n "$INSTRUCTIONS_FILE" && -f "$INSTRUCTIONS_FILE" ]]; then
  "${CMD[@]}" < "$INSTRUCTIONS_FILE" || exit_code=$?
else
  "${CMD[@]}" || exit_code=$?
fi

END_TS=$(date -Iseconds)
echo "[leaf plan] Finished: $END_TS (exit $exit_code)"

if [[ -n "${TMUX_PANE:-}" ]]; then
  tmux pipe-pane -t "${TMUX_PANE}"
fi

if [[ -n "$SUMMARY_FILE" ]]; then
  {
    echo "# leaf plan :: $RUN_LABEL"
    echo "- Run ID: $RUN_KEY"
    echo "- Started: $START_TS"
    echo "- Finished: $END_TS"
    echo "- Exit Code: $exit_code"
    echo ""
    echo "## Instructions"
    if [[ -n "$INSTRUCTIONS_FILE" && -f "$INSTRUCTIONS_FILE" ]]; then
      cat "$INSTRUCTIONS_FILE"
    else
      echo "(none provided)"
    fi
    echo ""
    echo "## Command"
    printf '`'
    printf '%q ' "${CMD[@]}"
    printf '`\n'
  } >"$SUMMARY_FILE"
fi

echo "$exit_code" >"$EXIT_CODE_FILE"
tmux wait-for -S "$WAIT_KEY"
exit "$exit_code"
