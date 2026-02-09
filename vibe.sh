#!/bin/bash
# Vibe Code - Claude-only autonomous loop

set -e
set -o pipefail

MAX_ITERATIONS=10
ACK_UNSAFE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    -h|--help)
      echo "Usage: ./vibe.sh [--acknowledge-unsafe] [max_iterations]"
      exit 0
      ;;
    --acknowledge-unsafe)
      ACK_UNSAFE=true
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      else
        echo "Error: Invalid argument '$1'. Expected a max iteration count."
        exit 1
      fi
      ;;
  esac
  shift
done

if ! command -v claude >/dev/null 2>&1; then
  echo "Error: Claude CLI not found. See README.md (Vibe Code section) for install steps."
  exit 1
fi

if [ "$ACK_UNSAFE" != "true" ]; then
  echo "Error: Vibe uses --dangerously-skip-permissions. Re-run with --acknowledge-unsafe to proceed."
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_FILE="$SCRIPT_DIR/VIBE.md"
TODO_FILE="$SCRIPT_DIR/todo.md"

if [ ! -f "$PROMPT_FILE" ]; then
  echo "Error: Missing VIBE.md prompt file."
  exit 1
fi

if [ ! -f "$TODO_FILE" ]; then
  echo "Error: Missing todo.md. Create it using the format in VIBE.md."
  exit 1
fi

echo "Starting Vibe Code - Max iterations: $MAX_ITERATIONS"
echo "Warning: --dangerously-skip-permissions bypasses confirmation prompts and grants full access. Use only in trusted repos."

for i in $(seq 1 "$MAX_ITERATIONS"); do
  echo ""
  echo "==============================================================="
  echo "  Vibe Iteration $i of $MAX_ITERATIONS"
  echo "==============================================================="

  set +e
  # Capture combined output so the completion tag is detected even if emitted on stderr.
  OUTPUT=$(claude --dangerously-skip-permissions --print < "$PROMPT_FILE" 2>&1)
  CLAUDE_STATUS=$?
  set -e

  if [ "$CLAUDE_STATUS" -ne 0 ]; then
    echo "Warning: Claude CLI exited with status $CLAUDE_STATUS."
  fi

  printf '%s\n' "$OUTPUT"

  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "Vibe completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    exit 0
  fi

  echo "Iteration $i complete. Continuing..."
  # Brief pause to avoid a tight loop and give the CLI a moment between runs.
  sleep 2
done

echo ""
echo "Vibe reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $TODO_FILE for status."
exit 1
