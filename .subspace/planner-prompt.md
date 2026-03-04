# Subspace Planner — Autonomous Project Completion Engine

Stack: **rust** | Iteration: **1**

## subspace.md (Existing Project State)

- Completion: 70%
- Pending tasks: 6
- Completed: 8

### Pending Tasks
- [ ] **🟡 Add unit tests for subspace_file parsing** — Test SubspaceFile::parse, pending_tasks, to_markdown round-trip.
- [ ] **🟡 Add unit tests for planner task parsing** — Test parse_tasks, is_complete, extract_files.
- [ ] **🟡 Add unit tests for project stack detection** — Test detect_stack with various project structures.
- [ ] **🟢 Wire up tmux-based CLI commands** — Connect swarm status/logs/steer/kill to actual tmux operations.
- [ ] **🟢 Add CI workflow** — GitHub Actions for cargo check, clippy, test.
- [ ] **🔵 Remove tokio-util dependency if unnecessary** — Check if tokio-util compat is still needed after ACP changes.

### Completed
- [x] Core CLI with all command definitions
- [x] ACP agent protocol integration
- [x] Git worktree agent isolation
- [x] Full plan/execute/merge orchestration loop
- [x] Dream mode with approval gate
- [x] Ratatui TUI dashboard
- [x] Local AI platform detection (Apple/Windows)
- [x] subspace.md persistent task management

---

## Mode: Tasks

subspace.md has **6** pending tasks. Work through them in priority order.

### Next Up
- 🟡 **🟡 Add unit tests for subspace_file parsing** — Test SubspaceFile::parse, pending_tasks, to_markdown round-trip.
- 🟡 **🟡 Add unit tests for planner task parsing** — Test parse_tasks, is_complete, extract_files.
- 🟡 **🟡 Add unit tests for project stack detection** — Test detect_stack with various project structures.
- 🟡 **🟢 Wire up tmux-based CLI commands** — Connect swarm status/logs/steer/kill to actual tmux operations.
- 🟡 **🟢 Add CI workflow** — GitHub Actions for cargo check, clippy, test.

### Instructions

1. **Read the current codebase** to understand the context for each task
2. **Pick the top parallelizable tasks** (no shared files between them)
3. **Run a quick audit** — do tests pass? Does it compile? Any blockers?
4. **Output tasks** for executor agents (use the exact task titles from subspace.md)
5. **Update subspace.md** — mark in-progress tasks as `[~]`, remove blockers

Do NOT invent new tasks unless the audit reveals critical issues not in the list.
If a task in subspace.md is already done (check the codebase), mark it `[x]`.

---

## Previous Progress

# Subspace Progress Log
Started: 2026-03-04 06:48:14 UTC
---


---

## Output Requirements

### 1. Update subspace.md

You MUST output an updated `subspace.md` file. Write it to disk.

Format:
```markdown
# Subspace

> Autonomous project completion file. Managed by Subspace AI swarm.

## Project
- **Name:** [name]
- **Description:** [what it does]
- **Completion:** [N]%

## Roadmap
- [x] Completed phase
- [ ] Next phase
- [ ] Future phase

## Tasks
### [ ] 🔴 Task Title
Description of what needs to be done.
- src/relevant/file.rs

### [x] 🟠 Already Done Task
What was accomplished.

## Completed
- Brief description of what's been completed overall

## Notes
Any important context for future iterations.
```

### 2. Task Output for Executors

If COMPLETE: `<subspace>COMPLETE</subspace>`

Otherwise, pick the highest-priority pending tasks and output them:

<subspace_tasks max_parallel="3">

### Task 1: [Exact title from subspace.md or new task]
#### Priority: 🔴/🟠/🟡/🟢/🔵
#### What
[Specific, actionable instructions for an AI developer]
#### Files Affected
- path/to/file1
- path/to/file2
#### Quality Checks
[Commands to verify]

</subspace_tasks>

## Rules

- Tasks must be PARALLELIZABLE (different files, no conflicts)
- Be SPECIFIC about what to implement and where
- Always update subspace.md on disk
- Mark tasks as [x] done and update completion %
- Don't redo completed work