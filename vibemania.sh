#!/bin/bash
# VibeMania - Two-phase autonomous AI development loop
# Usage: ./vibemania.sh [--tool claude|amp] [--project-dir PATH] [max_iterations]
#
# Phase A (Plan):   AI analyzes project goals and progress, outputs a specific plan
# Phase B (Execute): A fresh AI instance implements that plan
# Phase C (Review):  Progress is updated, loop checks for completion
#
# Based on ralph by snarktank (https://github.com/snarktank/ralph)

set -e

# --- Argument parsing ---
TOOL="claude"
MAX_ITERATIONS=10
PROJECT_DIR=""

print_help() {
  echo "Usage: ./vibemania.sh [--tool claude|amp] [--project-dir PATH] [max_iterations]"
  echo ""
  echo "Options:"
  echo "  --tool claude|amp    AI tool to use (default: claude)"
  echo "  --project-dir PATH   Target project directory (default: current directory)"
  echo "  -h, --help           Show this help message"
  echo ""
  echo "VibeMania runs a two-phase AI loop:"
  echo "  1. Planner AI reads your goals and decides what to do next"
  echo "  2. Executor AI implements the plan"
  echo "  3. Progress is logged and the loop repeats"
  echo ""
  echo "Requires goals.md in the project directory. See goals.md.example."
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
PLAN_FILE="$WORK_DIR/plan.md"
PLANNER_PROMPT_FILE="$WORK_DIR/planner-prompt.md"
EXECUTOR_PROMPT_FILE="$WORK_DIR/executor-prompt.md"
PROGRESS_FILE="$PROJECT_DIR/progress.md"
GOALS_FILE="$PROJECT_DIR/goals.md"
STACK_FILE="$WORK_DIR/stack.txt"

mkdir -p "$WORK_DIR"

# --- Language detection ---
detect_project_stack() {
  local dir="$1"
  local stack=""

  [ -f "$dir/package.json" ]       && stack="$stack,node"
  [ -f "$dir/tsconfig.json" ]      && stack="$stack,typescript"
  [ -f "$dir/Cargo.toml" ]         && stack="$stack,rust"
  [ -f "$dir/go.mod" ]             && stack="$stack,go"
  [ -f "$dir/requirements.txt" ]   && stack="$stack,python"
  [ -f "$dir/pyproject.toml" ]     && stack="$stack,python"
  [ -f "$dir/setup.py" ]           && stack="$stack,python"
  [ -f "$dir/Gemfile" ]            && stack="$stack,ruby"
  [ -f "$dir/pom.xml" ]            && stack="$stack,java-maven"
  [ -f "$dir/build.gradle" ]       && stack="$stack,java-gradle"
  [ -f "$dir/build.gradle.kts" ]   && stack="$stack,kotlin-gradle"
  [ -f "$dir/composer.json" ]      && stack="$stack,php"
  [ -f "$dir/mix.exs" ]            && stack="$stack,elixir"
  [ -f "$dir/pubspec.yaml" ]       && stack="$stack,dart-flutter"
  [ -f "$dir/Package.swift" ]      && stack="$stack,swift"
  [ -f "$dir/Makefile" ]           && stack="$stack,make"
  [ -f "$dir/CMakeLists.txt" ]     && stack="$stack,cmake"
  [ -f "$dir/Dockerfile" ]         && stack="$stack,docker"
  [ -f "$dir/docker-compose.yml" ] && stack="$stack,docker-compose"
  [ -f "$dir/.csproj" ] || find "$dir" -maxdepth 2 -name "*.csproj" -print -quit 2>/dev/null | grep -q . && stack="$stack,dotnet"
  [ -f "$dir/next.config.js" ] || [ -f "$dir/next.config.ts" ] || [ -f "$dir/next.config.mjs" ] && stack="$stack,nextjs"
  [ -f "$dir/vite.config.ts" ] || [ -f "$dir/vite.config.js" ] && stack="$stack,vite"
  [ -f "$dir/nuxt.config.ts" ] || [ -f "$dir/nuxt.config.js" ] && stack="$stack,nuxt"
  [ -f "$dir/svelte.config.js" ]   && stack="$stack,svelte"
  [ -f "$dir/angular.json" ]       && stack="$stack,angular"
  [ -f "$dir/tailwind.config.js" ] || [ -f "$dir/tailwind.config.ts" ] && stack="$stack,tailwind"

  # Remove leading comma, default to "unknown"
  stack="${stack#,}"
  echo "${stack:-unknown}"
}

# --- Build planner prompt ---
build_planner_prompt() {
  local goals_file="$1"
  local progress_file="$2"
  local stack_file="$3"
  local iteration="$4"

  local template stack goals progress
  template=$(cat "$SCRIPT_DIR/prompts/planner.md")
  stack=$(cat "$stack_file" 2>/dev/null || echo "unknown")
  goals=$(cat "$goals_file")
  progress=$(cat "$progress_file" 2>/dev/null || echo "No progress yet.")

  # Replace simple placeholders
  local prompt
  prompt=$(echo "$template" | sed "s|{{ITERATION}}|$iteration|g" | sed "s|{{STACK}}|$stack|g")

  # Append context blocks
  printf '%s\n\n## Current Goals\n\n%s\n\n## Progress So Far\n\n%s\n' "$prompt" "$goals" "$progress"
}

# --- Build executor prompt ---
build_executor_prompt() {
  local plan_file="$1"
  local stack_file="$2"
  local progress_file="$3"

  local template stack plan
  template=$(cat "$SCRIPT_DIR/prompts/executor.md")
  stack=$(cat "$stack_file" 2>/dev/null || echo "unknown")
  plan=$(cat "$plan_file")

  local prompt
  prompt=$(echo "$template" | sed "s|{{STACK}}|$stack|g")

  printf '%s\n\n## The Plan To Execute\n\n%s\n' "$prompt" "$plan"
}

# --- Pre-flight checks ---
if [ ! -f "$GOALS_FILE" ]; then
  echo "Error: Missing goals.md in $PROJECT_DIR"
  echo ""
  echo "Create a goals.md describing what you want to build."
  echo "See goals.md.example for the format."
  exit 1
fi

# Detect project stack
DETECTED_STACK=$(detect_project_stack "$PROJECT_DIR")
echo "$DETECTED_STACK" > "$STACK_FILE"

# Initialize progress file if needed
if [ ! -f "$PROGRESS_FILE" ]; then
  echo "# VibeMania Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

echo "============================================"
echo "  VibeMania"
echo "============================================"
echo "Tool:       $TOOL"
echo "Project:    $PROJECT_DIR"
echo "Stack:      $DETECTED_STACK"
echo "Iterations: $MAX_ITERATIONS"
echo "============================================"
echo ""

# --- Main Loop ---
for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Iteration $i of $MAX_ITERATIONS"
  echo "==============================================================="

  echo "$i" > "$WORK_DIR/iteration.txt"

  # ---- PHASE A: PLANNING ----
  echo ""
  echo "--- Phase A: Planning (asking AI what to do next) ---"
  echo ""

  PLANNER_PROMPT=$(build_planner_prompt "$GOALS_FILE" "$PROGRESS_FILE" "$STACK_FILE" "$i")
  echo "$PLANNER_PROMPT" > "$PLANNER_PROMPT_FILE"

  if [[ "$TOOL" == "claude" ]]; then
    PLAN_OUTPUT=$(claude --dangerously-skip-permissions --print < "$PLANNER_PROMPT_FILE" 2>&1 | tee /dev/stderr) || true
  else
    PLAN_OUTPUT=$(cat "$PLANNER_PROMPT_FILE" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  fi

  # Save plan for Phase B and for debugging
  echo "$PLAN_OUTPUT" > "$PLAN_FILE"

  # Check if planner says we are done
  if echo "$PLAN_OUTPUT" | grep -q "<vibemania>COMPLETE</vibemania>"; then
    echo ""
    echo "============================================"
    echo "  VibeMania: All goals achieved!"
    echo "  Completed at iteration $i of $MAX_ITERATIONS"
    echo "============================================"
    exit 0
  fi

  # ---- PHASE B: EXECUTION ----
  echo ""
  echo "--- Phase B: Execution (implementing the plan) ---"
  echo ""

  EXECUTOR_PROMPT=$(build_executor_prompt "$PLAN_FILE" "$STACK_FILE" "$PROGRESS_FILE")
  echo "$EXECUTOR_PROMPT" > "$EXECUTOR_PROMPT_FILE"

  if [[ "$TOOL" == "claude" ]]; then
    EXEC_OUTPUT=$(claude --dangerously-skip-permissions --print < "$EXECUTOR_PROMPT_FILE" 2>&1 | tee /dev/stderr) || true
  else
    EXEC_OUTPUT=$(cat "$EXECUTOR_PROMPT_FILE" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
  fi

  echo ""
  echo "--- Iteration $i complete ---"
  sleep 2
done

echo ""
echo "============================================"
echo "  VibeMania reached max iterations ($MAX_ITERATIONS)"
echo "  Check progress.md for status."
echo "============================================"
exit 1
