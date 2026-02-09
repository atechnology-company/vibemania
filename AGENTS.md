# VibeMania Agent Instructions

## Overview

VibeMania is a two-phase autonomous AI development loop. A planner AI decides what to do next, then an executor AI implements it. Each iteration spawns fresh AI instances with clean context.

## Commands

```bash
# Run VibeMania with Claude Code (default)
./vibemania.sh

# Run VibeMania with Amp
./vibemania.sh --tool amp

# Run with custom iterations and project directory
./vibemania.sh --tool claude --project-dir ./my-project 20

# Build the macOS app
cd vibe-gui && xcodegen generate && open VibeMania.xcodeproj
```

## Key Files

- `vibemania.sh` - The main two-phase loop script
- `prompts/planner.md` - Prompt template for the planning phase
- `prompts/executor.md` - Prompt template for the execution phase
- `goals.md.example` - Template for project goals
- `vibe-gui/` - Native macOS SwiftUI app for managing runs

## Patterns

- Each iteration spawns fresh AI instances (planner + executor) with clean context
- The planner reads `goals.md` and `progress.md` to decide what to work on
- The executor receives a specific plan and implements it
- Memory persists via git history and `progress.md`
- The detected tech stack is passed to both phases for appropriate quality checks
- Goals should be small enough that each can be achieved in 1-3 iterations
