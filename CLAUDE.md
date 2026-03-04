# CLAUDE.md — VibeMania Rust CLI

## What This Is
A Rust CLI tool for orchestrating AI coding agent swarms via tmux. Based on the existing bash scripts (`vibemania.sh`, `vibemania-swarm.sh`) in this repo, but rewritten as a proper Rust CLI that can be controlled programmatically.

## Architecture

### Core Concept
VibeMania manages Claude Code (or other AI coding agents) instances running in tmux sessions. It implements a plan/execute loop where:
1. A **planner** agent analyzes goals and creates tasks
2. **Executor** agents implement tasks in parallel (each in its own tmux pane)
3. Progress is tracked, conflicts detected, and results merged

### Key Design Goals
- **tmux-native**: Each agent runs in a named tmux session/window/pane
- **Controllable**: Another AI (like OpenClaw's Claw) can spawn, monitor, steer, and kill agents via CLI commands
- **Swarm-capable**: Run N parallel executors on independent tasks
- **Observable**: Stream logs, check status, detect file conflicts in real-time

### CLI Commands to Implement

```
vibemania init                          # Initialize project (create goals.md, .vibemania/)
vibemania plan [--project-dir PATH]     # Run planner phase only, output tasks
vibemania run [--max-iter N] [--parallel N] [--project-dir PATH]  # Full plan/execute loop
vibemania swarm launch [--agents N] [--project-dir PATH]  # Launch parallel agent swarm
vibemania swarm status                  # Show all running agents, their tasks, progress
vibemania swarm logs [agent-id]         # Stream/tail logs from specific agent
vibemania swarm steer [agent-id] "msg"  # Send instruction to specific agent
vibemania swarm kill [agent-id|--all]   # Kill specific or all agents
vibemania status                        # Overall project status (progress, conflicts)
vibemania conflicts                     # Show file conflicts between agents
```

### tmux Layout
```
Session: vibemania-{project-name}
├── Window 0: "planner"     - Planner agent
├── Window 1: "executor-1"  - Executor agent 1
├── Window 2: "executor-2"  - Executor agent 2
└── Window N: "executor-N"  - Executor agent N
```

### Dependencies to use
- `clap` for CLI parsing (derive API)
- `tokio` for async runtime
- `serde` + `serde_json` for config/state
- `colored` or `crossterm` for terminal output
- Standard `std::process::Command` for tmux interaction (no need for a tmux crate)

### File Structure
```
src/
├── main.rs          # CLI entry point with clap
├── cli.rs           # CLI argument definitions
├── tmux.rs          # tmux session/pane management
├── agent.rs         # Agent lifecycle (spawn, monitor, kill)
├── planner.rs       # Planner phase logic
├── executor.rs      # Executor phase logic  
├── swarm.rs         # Swarm orchestration (parallel agents)
├── project.rs       # Project detection, goals, progress
├── conflict.rs      # File conflict detection
└── config.rs        # Configuration and state persistence
```

### State File (.vibemania/state.json)
```json
{
  "session_name": "vibemania-myproject",
  "project_dir": "/path/to/project",
  "stack": "rust",
  "agents": [
    {
      "id": "planner-1",
      "role": "planner",
      "tmux_window": "planner",
      "status": "completed",
      "started_at": "...",
      "task": null
    },
    {
      "id": "executor-1", 
      "role": "executor",
      "tmux_window": "executor-1",
      "status": "running",
      "started_at": "...",
      "task": {"title": "Add auth module", "files": ["src/auth.rs"]}
    }
  ],
  "iteration": 3,
  "conflicts": []
}
```

### How tmux Control Works
```rust
// Spawn agent in new tmux window
tmux new-window -t vibemania-proj -n executor-1 "claude --dangerously-skip-permissions --print < /tmp/prompt.md"

// Capture output (for monitoring)
tmux capture-pane -t vibemania-proj:executor-1 -p

// Send keys to agent
tmux send-keys -t vibemania-proj:executor-1 "additional instruction" Enter

// Kill agent
tmux send-keys -t vibemania-proj:executor-1 C-c
```

### Existing Resources
- `vibemania.sh` — single-agent plan/execute loop (reference implementation)
- `vibemania-swarm.sh` — parallel swarm version (reference)
- `prompts/planner.md` — planner prompt template
- `prompts/executor.md` — executor prompt template
- These bash scripts work but are the reference — the Rust CLI should be a clean rewrite using the same concepts

### Quality Checks
- `cargo check` must pass
- `cargo clippy` must pass  
- `cargo build --release` must succeed
- Code should be well-structured and idiomatic Rust

### Notes
- The tool defaults to `claude` as the AI backend but should support `--tool amp` flag too
- Prompts are loaded from `prompts/` directory relative to the project or from a default location
- Progress tracking via `progress.md` (append-only, same as bash version)
- Goals from `goals.md` in project root
- All tmux session names should be namespaced to avoid collisions
