# =============================================================================
# Functions
# =============================================================================

# cdx - Codex wrapper with update capability
cdx() {
  if [[ "$1" == "update" ]]; then
    npm install -g @openai/codex@latest
  else
    codex \
      -m gpt-5-codex \
      -c model_reasoning_summary_format=experimental \
      --search "$@"
  fi
}

# glowf - Find and view markdown files with glow
glowf() {
  local file
  file=$(find . -type f -name '*.md' | fzf --preview 'glow --style=dark --pager {}')
  [ -n "$file" ] && glow "$file"
}

# tb - Smart taskbook wrapper (uses local .taskbook if present)
tb() {
  if [[ -d .taskbook ]]; then
    command tb --storage-dir ./.taskbook "$@"
  else
    command tb "$@"
  fi
}

# git - Wrapper with retry logic for index.lock issues using attempt CLI
git() {
  if (( $+commands[attempt] )); then
    command attempt git "$@"
  else
    command git "$@"
  fi
}

# codex_safe - Run codex with confirmation before executing
codex_safe() {
  cmd="$(codex "$@" | tee /dev/tty)"
  base="${cmd%% *}"

  if [[ "$base" == "bd" ]]; then
    eval "$cmd"
  else
    read -r -p "Run this command? [y/N] " resp
    [[ "$resp" == "y" ]] && eval "$cmd"
  fi
}
