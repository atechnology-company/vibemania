# vibemania (archive -- moved to [unthinkclaw](https://github.com/undivisible/unthinkclaw))

vibemania is a two-phase autonomous AI development loop. Instead of running the same prompt repeatedly, vibemania uses one AI to **plan** what to do next, then feeds that plan to a second AI to **execute** it. Memory persists across iterations via `progress.md` and git history.

## Credits

Based on [snarktank/ralph](https://github.com/snarktank/ralph) by [Geoffrey Huntley](https://ghuntley.com/ralph/) and [Ryan Carson](https://x.com/ryancarson). vibemania extends the Ralph pattern with a two-phase plan/execute loop and automatic language detection.
## How It Works

Each iteration has two phases:

1. **Planner AI** reads your `goals.md`, the `progress.md` log, and the codebase. It decides the single most impactful next change and outputs a detailed plan.
2. **Executor AI** receives that plan in a fresh session and implements it -- writes code, runs quality checks, commits, and logs progress.

The loop repeats until all goals are achieved.

```
goals.md + progress.md
        |
    [Planner AI]  -->  "Add priority badges to task cards"
        |
    [Executor AI]  -->  writes code, runs tests, commits
        |
    progress.md updated
        |
    (repeat until done)
```

## Prerequisites

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) (`npm install -g @anthropic-ai/claude-code`) or [Amp CLI](https://ampcode.com)
- `jq` installed (`brew install jq` on macOS)
- A git repository for your project

## Quick Start

### 1. Create goals.md

Create a `goals.md` file in your project root describing what you want to build:

```markdown
# Project Goals

## What We Are Building
A task management API with priority levels and filtering.

## Specific Goals
- [ ] Add priority field (high/medium/low) to tasks table
- [ ] Create GET /api/tasks endpoint with priority filter
- [ ] Add priority badge component to task cards
- [ ] Add priority filter dropdown to task list

## Constraints
- Use existing Express router pattern
- Follow existing React component patterns

## Quality Requirements
- npm run typecheck must pass
- npm test must pass
```

See `goals.md.example` for the full template.

### 2. Run VibeMania

```bash
# Copy vibemania.sh to your project (or run from the vibemania directory)
./vibemania.sh
```

Options:

```bash
./vibemania.sh --tool claude     # Use Claude Code (default)
./vibemania.sh --tool amp        # Use Amp
./vibemania.sh 20                # Run up to 20 iterations (default: 10)
./vibemania.sh --project-dir ./my-app   # Target a specific directory
```

VibeMania will:
1. Detect your project's tech stack automatically
2. Ask the Planner AI what to work on next
3. Feed the plan to the Executor AI to implement
4. Log progress and repeat until all goals are met

## Language Detection

VibeMania automatically detects your project's tech stack by checking for:

| Marker File | Detected Stack |
|---|---|
| `package.json` | Node.js |
| `tsconfig.json` | TypeScript |
| `Cargo.toml` | Rust |
| `go.mod` | Go |
| `requirements.txt` / `pyproject.toml` | Python |
| `Gemfile` | Ruby |
| `pom.xml` / `build.gradle` | Java |
| `composer.json` | PHP |
| `mix.exs` | Elixir |
| `Package.swift` | Swift |
| `pubspec.yaml` | Dart/Flutter |
| `next.config.js` | Next.js |
| `vite.config.ts` | Vite |
| And more... | |

The detected stack is passed to both the planner and executor so they suggest and run appropriate quality checks (e.g., `cargo test` for Rust, `npm run lint` for Node).

## Key Files

| File | Purpose |
|---|---|
| `vibemania.sh` | Main two-phase loop script |
| `prompts/planner.md` | Prompt template for the planning phase |
| `prompts/executor.md` | Prompt template for the execution phase |
| `goals.md` | Your project goals (create from `goals.md.example`) |
| `goals.md.example` | Template for goals file |
| `progress.md` | Append-only log of what was done each iteration |
| `.vibemania/` | Working directory (plans, detected stack, temp files) |
| `skills/prd/` | Skill for generating PRDs |
| `skills/vibemania/` | Skill for setting up goals.md |
| `vibe-gui/` | Native macOS SwiftUI app for managing VibeMania runs |

## Skills

VibeMania includes skills for AI coding tools:

- `/prd` - Generate a Product Requirements Document
- `/vibemania` - Set up `goals.md` for your project interactively

### Install skills globally

For Claude Code:
```bash
cp -r skills/prd ~/.claude/skills/
cp -r skills/vibemania ~/.claude/skills/
```

For Amp:
```bash
cp -r skills/prd ~/.config/amp/skills/
cp -r skills/vibemania ~/.config/amp/skills/
```

## macOS App

The `vibe-gui/` directory contains a native macOS SwiftUI application for managing VibeMania runs. It provides:

- Dashboard for monitoring active agent runs
- Project management with sidebar navigation
- Real-time log viewer for agent output
- Agent lifecycle management (start/stop/monitor)

Requires macOS 14.0+ and Xcode. Open `vibe-gui/VibeMania.xcodeproj` to build and run.

## How VibeMania Differs from Ralph

| | Ralph | VibeMania |
|---|---|---|
| **Loop type** | Single-phase: same static prompt every iteration | Two-phase: planner decides, executor implements |
| **Task source** | `prd.json` with pre-defined user stories | `goals.md` with freeform goals; AI plans dynamically |
| **What to work on** | AI reads PRD and picks next incomplete story | Planner AI analyzes the full project and decides |
| **Language awareness** | None - prompt is static | Auto-detects project stack and adapts quality checks |
| **Progress tracking** | `progress.txt` + `prd.json` status | `progress.md` read by planner each iteration |

## Debugging

```bash
# See what the planner suggested last
cat .vibemania/plan.md

# See progress so far
cat progress.md

# See detected tech stack
cat .vibemania/stack.txt

# Check git history
git log --oneline -10
```

## License

Licensed under the ISC License. See [LICENSE](LICENSE) for details.

## References

- [snarktank/ralph](https://github.com/snarktank/ralph) - The original autonomous AI agent loop
- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
- [Amp documentation](https://ampcode.com/manual)
