# VibeMania

Two-phase autonomous AI development loop. A planner AI decides what to do, an executor AI implements it.

## Commands

```bash
# Run VibeMania
./vibemania.sh [--tool claude|amp] [--project-dir PATH] [max_iterations]

# Build the macOS app
cd vibe-gui && xcodegen generate && open VibeMania.xcodeproj
```

## Architecture

- `vibemania.sh` - Main two-phase loop script
- `prompts/planner.md` - Planning phase prompt template
- `prompts/executor.md` - Execution phase prompt template
- `goals.md` - User-defined project goals (create from `goals.md.example`)
- `progress.md` - Append-only iteration log
- `.vibemania/` - Working directory (gitignored)
- `skills/` - AI tool skills (PRD generation, goals setup)
- `vibe-gui/` - Native macOS SwiftUI app for managing runs

## How It Works

1. User creates `goals.md` with project goals
2. `vibemania.sh` detects the project stack and starts the loop
3. Each iteration: Planner AI analyzes goals + progress, outputs a plan
4. Executor AI receives the plan and implements it in a fresh session
5. Progress is logged, loop repeats until all goals are achieved

## Based On

[snarktank/ralph](https://github.com/snarktank/ralph) - the original autonomous AI agent loop.
