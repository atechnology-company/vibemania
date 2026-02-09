#!/bin/bash
# VibeMania Swarm - Parallel multi-agent execution
# Usage: ./vibemania-swarm.sh [--tool claude|amp] [--project-dir PATH] [--session-id ID] [max_iterations]
#
# Phase A (Plan):    Planner AI generates multiple independent tasks
# Phase B (Execute): Multiple executor AIs run tasks in parallel
# Phase C (Merge):   Results are aggregated and conflicts detected
#
# Based on vibemania.sh

set -e

# --- Argument parsing ---
TOOL="claude"
MAX_ITERATIONS=10
PROJECT_DIR=""
SESSION_ID="default"

print_help() {
  echo "Usage: ./vibemania-swarm.sh [--tool claude|amp] [--project-dir PATH] [--session-id ID] [max_iterations]"
  echo ""
  echo "Options:"
  echo "  --tool claude|amp    AI tool to use (default: claude)"
  echo "  --project-dir PATH   Target project directory (default: current directory)"
  echo "  --session-id ID      Conversation session ID for tracking"
  echo "  -h, --help           Show this help message"
  echo ""
  echo "VibeMania Swarm runs a parallel multi-agent AI loop:"
  echo "  1. Planner AI reads goals and generates multiple independent tasks"
  echo "  2. Multiple Executor AIs implement tasks in parallel"
  echo "  3. Results are merged and conflicts are detected"
  echo ""
  echo "Requires goals.md in the project directory."
  exit 0
}

while [[ $# -gt 0 ]]; do
  case $1 in
    -h|--help)
      print_help
      ;;
    --tool)
      TOOL="$2"
      shift 2
      ;;
    --tool=*)
      TOOL="${1#*=}"
      shift
      ;;
    --project-dir)
      PROJECT_DIR="$2"
      shift 2
      ;;
    --project-dir=*)
      PROJECT_DIR="${1#*=}"
      shift
      ;;
    --session-id)
      SESSION_ID="$2"
      shift 2
      ;;
    --session-id=*)
      SESSION_ID="${1#*=}"
      shift
      ;;
    *)
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
      else
        echo "Error: Unknown argument '$1'. Use -h for help."
        exit 1
      fi
      shift
      ;;
  esac
done

# Validate tool choice
if [[ "$TOOL" != "claude" && "$TOOL" != "amp" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'claude' or 'amp'."
  exit 1
fi

# Check tool is installed
if [[ "$TOOL" == "claude" ]] && ! command -v claude >/dev/null 2>&1; then
  echo "Error: Claude CLI not found. Install: npm install -g @anthropic-ai/claude-code"
  exit 1
fi

if [[ "$TOOL" == "amp" ]] && ! command -v amp >/dev/null 2>&1; then
  echo "Error: Amp CLI not found. Install from https://ampcode.com"
  exit 1
fi

# --- Setup paths ---
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -z "$PROJECT_DIR" ]; then
  PROJECT_DIR="$(pwd)"
fi
PROJECT_DIR="$(cd "$PROJECT_DIR" && pwd)"

WORK_DIR="$PROJECT_DIR/.vibemania"
TASKS_FILE="$WORK_DIR/tasks.json"
PLAN_FILE="$WORK_DIR/plan.md"
PLANNER_PROMPT_FILE="$WORK_DIR/planner-prompt.md"
PROGRESS_FILE="$PROJECT_DIR/progress.md"
GOALS_FILE="$PROJECT_DIR/goals.md"
STACK_FILE="$WORK_DIR/stack.txt"

mkdir -p "$WORK_DIR"

# Source helper functions from vibemania.sh
source_vibemania_functions() {
  # Detect project stack (copied from vibemania.sh)
  local dir="$1"
  local stack=""

  [ -f "$dir/package.json" ]       && stack="$stack,node"
  [ -f "$dir/tsconfig.json" ]      && stack="$stack,typescript"
  [ -f "$dir/Cargo.toml" ]         && stack="$stack,rust"
  [ -f "$dir/go.mod" ]             && stack="$stack,go"
  [ -f "$dir/requirements.txt" ]   && stack="$stack,python"
  [ -f "$dir/pyproject.toml" ]     && stack="$stack,python"
  [ -f "$dir/Package.swift" ]      && stack="$stack,swift"
  # ... (add more as needed)

  stack="${stack#,}"
  echo "${stack:-unknown}"
}

DETECTED_STACK=$(source_vibemania_functions "$PROJECT_DIR")
echo "$DETECTED_STACK" > "$STACK_FILE"

# Pre-flight checks
if [ ! -f "$GOALS_FILE" ]; then
  echo "Error: Missing goals.md in $PROJECT_DIR"
  exit 1
fi

# Initialize progress file if needed
if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# VibeMania Swarm Progress Log" > "$PROGRESS_FILE"
  echo "Session: $SESSION_ID" >> "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

echo "============================================"
echo "  VibeMania Swarm"
echo "============================================"
echo "Tool:       $TOOL"
echo "Project:    $PROJECT_DIR"
echo "Stack:      $DETECTED_STACK"
echo "Session:    $SESSION_ID"
echo "Iterations: $MAX_ITERATIONS"
echo "============================================"
echo ""

# --- Helper: Build planner prompt ---
build_planner_prompt() {
  local template goals progress stack
  template=$(cat "$SCRIPT_DIR/prompts/planner.md")
  stack=$(cat "$STACK_FILE" 2>/dev/null || echo "unknown")
  goals=$(cat "$GOALS_FILE")
  progress=$(cat "$PROGRESS_FILE" 2>/dev/null || echo "No progress yet.")

  local prompt
  prompt=$(echo "$template" | sed "s|{{ITERATION}}|$1|g" | sed "s|{{STACK}}|$stack|g")

  printf '%s\n\n## Current Goals\n\n%s\n\n## Progress So Far\n\n%s\n' "$prompt" "$goals" "$progress"
}

# --- Helper: Parse tasks from planner output ---
parse_tasks_from_plan() {
  local plan_content="$1"

  # Check if planner output contains multi-task format
  if echo "$plan_content" | grep -q "<vibemania_tasks"; then
    echo "INFO: Detected multi-task plan" >&2

    # Extract max_parallel attribute
    local max_parallel
    max_parallel=$(echo "$plan_content" | grep -oP 'max_parallel="\K[0-9]+' || echo "3")

    # Split tasks by "### Task" pattern
    local task_num=1
    while IFS= read -r line; do
      if [[ "$line" =~ ^###\ Task\ [0-9]+: ]]; then
        if [[ $task_num -gt 1 ]]; then
          # Close previous task
          echo "}" >> "$TASKS_FILE.tmp"
        fi

        # Start new task
        local title=$(echo "$line" | sed 's/^### Task [0-9]*: //')
        echo "{\"id\": $task_num, \"title\": \"$title\", \"planFile\": \"plan-$task_num.md\", \"status\": \"pending\"}" >> "$TASKS_FILE.tmp"

        # Save task content to individual plan file
        task_content=""
        task_num=$((task_num + 1))
      fi
    done < <(echo "$plan_content" | grep "^###")

    # Convert to JSON array
    echo "[" > "$TASKS_FILE"
    cat "$TASKS_FILE.tmp" | sed '${s/$/,/;s/,$/}/}' >> "$TASKS_FILE"
    echo "]" >> "$TASKS_FILE"
    rm -f "$TASKS_FILE.tmp"

    return $((task_num - 1)) # Return task count
  else
    echo "INFO: Single task plan, will run serially" >&2
    return 1
  fi
}

# --- Main Loop ---
for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Iteration $i of $MAX_ITERATIONS"
  echo "==============================================================="

  echo "$i" > "$WORK_DIR/iteration.txt"

  # ---- PHASE A: PLANNING ----
  echo ""
  echo "--- Phase A: Planning (generating tasks) ---"
  echo ""

  PLANNER_PROMPT=$(build_planner_prompt "$i")
  echo "$PLANNER_PROMPT" > "$PLANNER_PROMPT_FILE"

  if [[ "$TOOL" == "claude" ]]; then
    PLAN_OUTPUT=$(claude --dangerously-skip-permissions --print < "$PLANNER_PROMPT_FILE" 2>&1 | tee /dev/stderr) || true
  else
    PLAN_OUTPUT=$(cat "$PLANNER_PROMPT_FILE" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  fi

  echo "$PLAN_OUTPUT" > "$PLAN_FILE"

  # Check if planner says we are done
  if echo "$PLAN_OUTPUT" | grep -q "<vibemania>COMPLETE</vibemania>"; then
    echo ""
    echo "============================================"
    echo "  VibeMania Swarm: All goals achieved!"
    echo "  Completed at iteration $i of $MAX_ITERATIONS"
    echo "============================================"
    exit 0
  fi

  # Parse tasks from plan
  parse_tasks_from_plan "$PLAN_OUTPUT"
  task_count=$?

  if [[ $task_count -gt 1 ]]; then
    echo ""
    echo "--- Phase B: Parallel Execution ($task_count tasks) ---"
    echo ""

    # Launch parallel executors
    pids=()
    for task_id in $(seq 1 $task_count); do
      echo "Launching executor for Task $task_id..."

      # Create isolated plan file for this task
      # (In full implementation, split plan content by task)
      cp "$PLAN_FILE" "$WORK_DIR/plan-$task_id.md"

      # Launch executor in background
      (
        # Build executor prompt
        local executor_prompt_file="$WORK_DIR/executor-prompt-$task_id.md"
        template=$(cat "$SCRIPT_DIR/prompts/executor.md")
        plan=$(cat "$WORK_DIR/plan-$task_id.md")
        stack=$(cat "$STACK_FILE")

        prompt=$(echo "$template" | sed "s|{{STACK}}|$stack|g")
        printf '%s\n\n## The Plan To Execute\n\n%s\n' "$prompt" "$plan" > "$executor_prompt_file"

        # Run executor
        if [[ "$TOOL" == "claude" ]]; then
          claude --dangerously-skip-permissions --print < "$executor_prompt_file" 2>&1 | tee "$WORK_DIR/executor-$task_id.log"
        else
          cat "$executor_prompt_file" | amp --dangerously-allow-all 2>&1 | tee "$WORK_DIR/executor-$task_id.log"
        fi

        # Mark task complete
        echo "Task $task_id completed" > "$WORK_DIR/task-$task_id.done"
      ) &
      pids+=($!)
    done

    # Wait for all executors to complete
    echo "Waiting for all executors to finish..."
    for pid in "${pids[@]}"; do
      wait "$pid" || echo "Warning: Executor $pid failed"
    done

    echo ""
    echo "--- Phase C: Merge & Conflict Detection ---"
    echo ""

    # Merge progress files with flock for safety
    for task_id in $(seq 1 $task_count); do
      if [[ -f "$WORK_DIR/executor-$task_id.log" ]]; then
        flock "$PROGRESS_FILE" -c "cat $WORK_DIR/executor-$task_id.log >> $PROGRESS_FILE"
      fi
    done

    # TODO: Detect file conflicts (check which files each task modified)
    echo "Conflict detection: TODO (future enhancement)"

  else
    echo ""
    echo "--- Phase B: Serial Execution (single task) ---"
    echo ""

    # Fall back to single executor
    executor_prompt_file="$WORK_DIR/executor-prompt.md"
    template=$(cat "$SCRIPT_DIR/prompts/executor.md")
    plan=$(cat "$PLAN_FILE")
    stack=$(cat "$STACK_FILE")

    prompt=$(echo "$template" | sed "s|{{STACK}}|$stack|g")
    printf '%s\n\n## The Plan To Execute\n\n%s\n' "$prompt" "$plan" > "$executor_prompt_file"

    if [[ "$TOOL" == "claude" ]]; then
      claude --dangerously-skip-permissions --print < "$executor_prompt_file" 2>&1 | tee /dev/stderr || true
    else
      cat "$executor_prompt_file" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr || true
    fi
  fi

  echo ""
  echo "--- Iteration $i complete ---"
  sleep 2
done

echo ""
echo "============================================"
echo "  VibeMania Swarm reached max iterations ($MAX_ITERATIONS)"
echo "  Check progress.md for status."
echo "============================================"
exit 1
