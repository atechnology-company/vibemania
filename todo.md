# VibeMania TODO & Roadmap

## ✅ Completed (Phase 1 - Codex-Inspired UI)

### Core Infrastructure
- [x] ChatMessage model with role-based messaging
- [x] AgentTask model for task tracking
- [x] ConversationSession model for thread management
- [x] ConversationStore for persistence
- [x] Extended Agent model with role, taskId, messages, filesModified

### UI Components
- [x] WelcomeView with "Let's build" interface
- [x] ChatView with message bubbles and input
- [x] AgentSwarmView for multi-agent visualization
- [x] ConversationListView for sidebar
- [x] ConversationView with HSplitView layout
- [x] ContentView with mode toggle (Conversations/Projects)
- [x] ConflictWarningBanner for file conflict alerts
- [x] Project path picker with file dialog

### Features
- [x] AgentManager swarm launch capability
- [x] Conflict detection infrastructure
- [x] Export conversation to markdown
- [x] Updated planner.md for multi-task output
- [x] vibemania-swarm.sh script for parallel execution (MVP)
- [x] Real-time agent status updates
- [x] Glassmorphism UI with hover effects

---

## 🚧 In Progress (Phase 2 - Parallel Execution)

### High Priority
- [ ] **Implement full task parsing from planner XML output**
  - Parse `<vibemania_tasks>` blocks from planner output
  - Extract individual task sections
  - Create AgentTask objects with filesAffected
  - Store tasks in ConversationSession.tasks array

- [ ] **Enhance vibemania-swarm.sh to split plan into separate task files**
  - Improve task extraction regex/logic
  - Create numbered plan files (plan-1.md, plan-2.md, etc.)
  - Spawn parallel executor processes with proper isolation
  - Implement flock-based progress.md locking
  - Test with deliberately conflicting tasks

- [ ] **Parse agent output to extract filesModified**
  - Add regex parsing in AgentManager pipe handlers
  - Look for git status changes or file edit patterns
  - Update Agent.filesModified array in real-time
  - Display in AgentSwarmCard

---

## 📋 Backlog (Phase 3 - Advanced Features)

### Multi-Agent Orchestration
- [ ] **Task dependency resolution**
  - Parse `<dependency>` tags from planner
  - Build task DAG (directed acyclic graph)
  - Execute tasks in topological order
  - Block dependent tasks until prerequisites complete

- [ ] **Dynamic agent scaling**
  - Monitor task completion velocity
  - Spawn additional executors if tasks finish quickly
  - Implement max_parallel limit enforcement
  - Add agent pool management

- [ ] **Inter-agent communication**
  - Shared state file (`.vibemania/agents-state.json`)
  - Message passing between agents
  - Coordination for shared resources
  - Leader election for conflicting decisions

### Conflict Resolution
- [ ] **Visual diff viewer for file conflicts**
  - Three-pane diff view (base, agent1, agent2)
  - Syntax highlighting
  - Manual merge conflict resolution UI
  - Accept left/right/both buttons

- [ ] **Automatic conflict resolution strategies**
  - Last-write-wins (with warning)
  - Semantic merge using git merge-file
  - AI-assisted conflict resolution (ask planner to decide)
  - Rollback mechanism for bad merges

- [ ] **Conflict prevention**
  - Pre-execution file locking
  - Task scheduling to avoid same-file modifications
  - Planner awareness of file assignments

### Task Management
- [ ] **Task status monitoring from tasks.json**
  - File watcher on `.vibemania/tasks.json`
  - Update GUI in real-time as tasks complete
  - Show task progress bars
  - Display task logs in separate panel

- [ ] **Task visualization**
  - Gantt chart for task timeline
  - Dependency graph viewer
  - Task completion percentage
  - Estimated time remaining

- [ ] **Task retry mechanism**
  - Automatic retry on failure
  - Exponential backoff
  - Max retry limit
  - Manual retry button in UI

### Enhanced UI/UX
- [ ] **Agent spawn animations**
  - Smooth card entrance animations
  - Pulse effect for active agents
  - Confetti on task completion
  - Error shake animation

- [ ] **Real-time collaboration indicators**
  - Show which files each agent is modifying
  - File lock indicators
  - Agent avatars on file icons
  - Live cursor positions (future: code editor integration)

- [ ] **Notification system**
  - macOS notifications for task completion
  - Sound effects for events
  - Badge count on app icon
  - Notification center integration

- [ ] **Keyboard shortcuts**
  - Cmd+N for new conversation
  - Cmd+E for export
  - Cmd+K for quick search
  - Cmd+R to restart failed agents

### Project Management
- [ ] **Project templates**
  - Pre-defined goal templates (REST API, React app, etc.)
  - Stack-specific defaults
  - Shareable template files
  - Template marketplace

- [ ] **Git integration**
  - Auto-commit after each iteration
  - Branch per conversation
  - PR creation from conversation
  - Commit message generation from tasks

- [ ] **Cloud sync**
  - Sync conversations across devices
  - Backup to iCloud/Dropbox
  - Share conversations via URL
  - Collaborative editing

---

## 🔬 Research & Experiments (Phase 4)

### AI Enhancements
- [ ] **Planner intelligence improvements**
  - Learn from past failures (RAG over progress.md)
  - Suggest optimal task parallelization
  - Predict task duration
  - Auto-detect code smells in goals

- [ ] **Executor specialization**
  - Frontend-specific executor (React/SwiftUI expert)
  - Backend-specific executor (API/database expert)
  - DevOps executor (Docker/CI/CD expert)
  - Route tasks to specialized agents

- [ ] **Quality assurance agent**
  - Separate agent that reviews code
  - Runs tests automatically
  - Suggests improvements
  - Creates issues for technical debt

### Performance & Scalability
- [ ] **Remote execution**
  - Execute agents on cloud VMs
  - Support for beefy machines
  - Cost estimation and limits
  - GPU acceleration for AI inference

- [ ] **Incremental execution**
  - Only re-run affected tasks on changes
  - Caching of intermediate results
  - Smart rebuild detection
  - Faster iteration cycles

- [ ] **Distributed execution**
  - Run multiple projects simultaneously
  - Share agent pool across projects
  - Load balancing
  - Resource quotas

### Developer Experience
- [ ] **VS Code extension**
  - Inline agent suggestions
  - Code lens for agent actions
  - Live progress in status bar
  - Quick actions from editor

- [ ] **CLI improvements**
  - Interactive TUI mode
  - Better progress indicators
  - Colorized output
  - Command history

- [ ] **Testing & CI integration**
  - GitHub Actions workflow
  - Pre-commit hooks
  - Automated quality checks
  - Performance benchmarks

---

## 🐛 Known Issues & Bugs

### Critical
- [ ] AgentManager.agentsForSession() always returns all agents (needs sessionId in Agent model)
- [ ] vibemania-swarm.sh task parsing is incomplete (regex needs work)
- [ ] No actual file conflict detection yet (filesModified not populated)

### Medium
- [ ] Conversation sessions not auto-saved during updates (only on add)
- [ ] No error handling for vibemania.sh not found
- [ ] Agent logs can grow unbounded (memory leak)
- [ ] No way to cancel a running conversation

### Low
- [ ] WelcomeView "Worktree" and "Cloud" buttons do nothing
- [ ] No dark mode support yet
- [ ] Sidebar doesn't auto-refresh on agent status changes
- [ ] Export markdown doesn't include agent logs

---

## 📚 Documentation Needed

- [ ] User guide for getting started
- [ ] Video tutorial for first conversation
- [ ] Architecture diagram for developers
- [ ] API documentation for extending VibeMania
- [ ] Troubleshooting guide
- [ ] Best practices for goal writing
- [ ] Example goals.md files for common projects

---

## 🎯 Metrics & Success Criteria

### Phase 2 Success Metrics
- [ ] Successfully run 3+ parallel agents on real project
- [ ] Detect and display file conflicts correctly
- [ ] Export conversation with full context
- [ ] < 5 second latency from plan to execution

### Phase 3 Success Metrics
- [ ] Resolve conflicts without user intervention 80% of time
- [ ] Complete typical web app in < 30 minutes
- [ ] Support 10+ parallel agents
- [ ] Zero data loss in conversation persistence

### Phase 4 Success Metrics
- [ ] 1000+ users on GitHub
- [ ] 90%+ successful task completion rate
- [ ] Average 5x speedup vs manual coding
- [ ] < 1% error rate in conflict resolution

---

## 🤝 Contributing

Want to help? Here are good first issues:

1. Add dark mode support
2. Implement keyboard shortcuts
3. Add more project templates
4. Improve error messages
5. Write tests for AgentManager
6. Create example goals.md files

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📝 Notes

- This TODO is a living document - update as we go
- Prioritize user value over technical perfection
- Ship early, iterate based on feedback
- Keep the UI simple and fast

**Last Updated:** 2026-02-09
