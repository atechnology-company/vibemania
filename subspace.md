# Subspace

> Autonomous project completion file. Managed by Subspace AI swarm.

## Project
- **Name:** Subspace (vibemania)
- **Description:** AI coding agent swarm orchestrator via tmux/ACP — manages parallel AI agents in isolated git worktrees
- **Completion:** 72%

## Roadmap
- [x] Core CLI scaffolding (clap, tokio, modules)
- [x] ACP integration for agent spawning
- [x] Git worktree isolation for parallel agents
- [x] Plan/execute/merge loop (swarm orchestration)
- [x] Dream mode (creative feature invention)
- [x] TUI dashboard with ratatui
- [x] Local AI backends (Apple Intelligence, Windows AI)
- [x] subspace.md task file management
- [ ] Zero compiler warnings and clean clippy
- [ ] Test suite for core modules
- [ ] Wire up tmux-based agent commands (status, logs, steer, kill)
- [ ] CI/CD pipeline
- [ ] Documentation and examples

## Tasks
### [x] 🔴 Fix all compiler warnings (55 warnings from cargo check)
Remove unused imports, fix unused variables, silence dead code warnings for planned-but-unwired modules.
- src/planner.rs
- src/merger.rs
- src/dream.rs
- src/tui.rs
- src/local/apple.rs
- src/local/windows.rs
- src/local/mod.rs
- src/main.rs
- src/tmux.rs
- src/agent.rs
- src/executor.rs
- src/swarm.rs
- src/project.rs
- src/conflict.rs
- src/worktree.rs
- src/acp.rs
- src/subspace_file.rs

### [~] 🔴 Fix all clippy warnings
Fix empty_line_after_outer_attr (5 files), manual_strip (subspace_file, dream), collapsible_if (planner), print_literal (swarm), filter_next (dream), collapsible_else_if (main).
- src/tmux.rs
- src/agent.rs
- src/executor.rs
- src/conflict.rs
- src/merger.rs
- src/subspace_file.rs
- src/planner.rs
- src/swarm.rs
- src/dream.rs
- src/main.rs

### [~] 🟡 Add unit tests for subspace_file parsing
Test SubspaceFile::parse, pending_tasks, to_markdown round-trip.
- src/subspace_file.rs

### [~] 🟡 Add unit tests for planner task parsing
Test parse_tasks, is_complete, extract_files.
- src/planner.rs

### [~] 🟡 Add unit tests for project stack detection
Test detect_stack with various project structures.
- src/project.rs

### [ ] 🟢 Wire up tmux-based CLI commands
Connect swarm status/logs/steer/kill to actual tmux operations.
- src/main.rs
- src/swarm.rs

### [ ] 🟢 Add CI workflow
GitHub Actions for cargo check, clippy, test.
- .github/workflows/ci.yml

### [-] 🔵 Remove tokio-util dependency if unnecessary
tokio-util compat is actively used in src/acp.rs for TokioAsyncReadCompatExt/TokioAsyncWriteCompatExt. Dependency is needed — skipped.
- Cargo.toml

## Completed
- Core CLI with all command definitions
- ACP agent protocol integration
- Git worktree agent isolation
- Full plan/execute/merge orchestration loop
- Dream mode with approval gate
- Ratatui TUI dashboard
- Local AI platform detection (Apple/Windows)
- subspace.md persistent task management

## Notes
- The tmux.rs, agent.rs, executor.rs, merger.rs, and conflict.rs modules contain complete implementations for the tmux-based agent control path. These are not yet wired into the main CLI dispatch but represent planned functionality. They are marked #[allow(dead_code)] to keep them available.
- The project uses ACP (Agent Client Protocol) as the primary agent backend, with tmux as the planned fallback/alternative.
- tokio-util is still needed — used in acp.rs for async stream compat layer.
- cargo clippy shows 16 non-dead-code lints that still need fixing (empty_line_after_outer_attr, manual_strip, collapsible_if, print_literal, filter_next, collapsible_else_if).
- No linker (`cc`) available in current env so `cargo test` can't link binaries, but `cargo check` confirms compilation.
