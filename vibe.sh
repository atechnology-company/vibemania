#!/bin/bash
# Vibe Code - Claude-only autonomous loop

set -e

MAX_ITERATIONS=10

while [[ $# -gt 0 ]]; do
  case $1 in
    -h|--help)
      echo "Usage: ./vibe.sh [max_iterations]"
      exit 0
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
  echo "Error: Claude Code CLI not found. Install with 'npm install -g @anthropic-ai/claude-code'."
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
  cat <<'EOF' > "$TODO_FILE"
# Vibe TODO

## Backlog
- [ ] Read README.md and any docs to understand project goals
- [ ] Scan the repo for TODO/FIXME markers or missing tasks
- [ ] Create a prioritized task list for the next steps

## In Progress
- [ ] (Move the active task here)

## Done
- [x] (Completed tasks go here)
EOF
fi

echo "Starting Vibe Code - Max iterations: $MAX_ITERATIONS"
echo "Warning: Vibe uses --dangerously-skip-permissions for autonomous operation."

for i in $(seq 1 "$MAX_ITERATIONS"); do
  echo ""
  echo "==============================================================="
  echo "  Vibe Iteration $i of $MAX_ITERATIONS"
  echo "==============================================================="

  OUTPUT=$(claude --dangerously-skip-permissions --print < "$PROMPT_FILE" 2>&1 | tee /dev/stderr) || true

  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "Vibe completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    exit 0
  fi

  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Vibe reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $TODO_FILE for status."
exit 1
