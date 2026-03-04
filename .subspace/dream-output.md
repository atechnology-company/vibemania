Let me explore the project thoroughly before dreaming.<dream_features>

### Feature 1: Speculative Execution — Branch Racing
**Wow Factor:** 10/10
**The Dream:** For any ambiguous or high-stakes task, spawn 2-3 agents that each take a *different approach* to the same problem — then automatically score the results by running tests, clippy, and a judge agent, and merge only the winner. Like CPU speculative execution, but for entire code strategies. A developer says "add caching" and gets back the best of three competing architectures, battle-tested.
**Why It's Remarkable:** Every other AI coding tool gives you one shot. This gives you a tournament. It turns the swarm's parallelism from "different tasks" into "competing solutions" — a fundamentally different use of multi-agent systems.
**How To Build It:**
1. Add a `SpecMode` variant to the executor phase in `swarm.rs` — when enabled, the planner outputs N alternative approaches per task instead of N different tasks
2. Create `src/speculative.rs`: takes one task + N approach descriptions, spawns N worktrees with identical goals but different strategy prompts
3. After all agents finish, run `cargo test`, `cargo clippy`, count warnings/errors per branch
4. Spawn a "judge" agent (Haiku) that reads all diffs + score metrics and picks the winner
5. Merge only the winning branch; archive losers as `subspace/spec-loser-{id}` branches for reference
6. Add `--speculative N` flag to `subspace run` (default off)
7. TUI integration: show agents in a "race" layout with live progress bars and a trophy icon on the winner
**Files To Create/Modify:**
- `src/speculative.rs` (new — race orchestrator, scoring, judge prompt)
- `src/cli.rs` (add `--speculative` flag)
- `src/swarm.rs` (call into speculative mode when flag set)
- `src/tui.rs` (race visualization widget)
**Creative Notes:** The judge prompt should include not just test results but *code quality heuristics* — cyclomatic complexity, number of unwrap() calls, whether error types are properly propagated. The losing branches aren't deleted — they become a "strategy library" that future planners can reference.

---

### Feature 2: Agent Replay — Time-Travel Debugging
**Wow Factor:** 9/10
**The Dream:** Every agent session is recorded as a replayable timeline — every prompt, every tool call, every file write, every decision point. Users can scrub through what an agent did like a video timeline, fork from any moment ("what if the agent had chosen a different approach at step 14?"), and re-execute from that point with new instructions. It's `git bisect` for AI reasoning.
**Why It's Remarkable:** AI agents are black boxes. When something goes wrong, you can't ask "why did you do that?" after the fact. Replay makes agent behavior inspectable, debuggable, and forkable. No other orchestrator offers this.
**How To Build It:**
1. Create `src/replay.rs` with a `SessionRecording` struct: ordered vec of `ReplayEvent` enum variants (PromptSent, TextChunk, ToolCallStart, ToolCallResult, FileWrite, FileRead, AgentComplete)
2. Hook into `acp.rs` — the `SubspaceClient` already intercepts all tool calls and text chunks; extend it to push each event into a `SessionRecording` with timestamps
3. Serialize recordings to `.subspace/recordings/{agent-id}-{timestamp}.json`
4. Add `subspace replay <recording-id>` CLI command that prints a step-by-step timeline with colored diffs for file writes
5. Add `subspace replay <recording-id> --fork-at <step>` that creates a new worktree at that point's file state and re-runs the agent with an amended prompt
6. TUI: add a replay viewer mode (bound to `r` key) that shows the timeline as a scrollable list with expandable tool call details
**Files To Create/Modify:**
- `src/replay.rs` (new — recording structs, serialization, timeline renderer, fork logic)
- `src/acp.rs` (instrument SubspaceClient to emit ReplayEvents)
- `src/cli.rs` (add `replay` subcommand with `--fork-at` flag)
- `src/tui.rs` (replay viewer widget)
**Creative Notes:** Recordings could be *diffed against each other* — "agent A took 47 tool calls to solve this, agent B took 12, here's where they diverged." This becomes a training signal for prompt optimization.

---

### Feature 3: Codebase DNA — Style Fingerprinting
**Wow Factor:** 8/10
**The Dream:** Before any agent writes a single line, Subspace scans the codebase and builds a "DNA profile" — naming conventions (snake_case vs camelCase frequency), error handling patterns (Result chains vs match blocks vs unwrap), module organization style, comment density, preferred crates, test patterns. This profile is automatically injected into every agent prompt so generated code is *stylistically indistinguishable* from the existing codebase.
**Why It's Remarkable:** The #1 complaint about AI-generated code is "it doesn't match our style." Every team has unwritten conventions. DNA extraction makes them explicit and enforceable without manual style guides.
**How To Build It:**
1. Create `src/dna.rs` with a `CodebaseDna` struct containing: naming_style, error_pattern, avg_function_length, comment_ratio, preferred_imports, test_structure, module_pattern
2. Implement `extract_dna(project_dir)` — walks `.rs` files (or `.ts`/`.py` depending on stack), uses regex/heuristic analysis to detect patterns: count `snake_case` vs `camelCase` identifiers, count `?` vs `.unwrap()` vs `match`, measure avg lines-per-function, detect test module patterns
3. Serialize to `.subspace/dna.json`; regenerate on `subspace init` or when stale (>24h or file count changed)
4. Format DNA as a concise "Style Guide" section automatically prepended to every executor and builder prompt
5. Add `subspace dna` CLI command to show the extracted profile in a pretty table
6. Add `subspace dna --diff` to compare the project's DNA against code written by agents (style drift detection)
**Files To Create/Modify:**
- `src/dna.rs` (new — extraction engine, pattern detection, prompt formatter)
- `src/cli.rs` (add `dna` subcommand)
- `src/swarm.rs` (load DNA and inject into executor prompts)
- `src/dream.rs` (inject DNA into builder prompts)
**Creative Notes:** For Rust projects, detect whether the codebase prefers `thiserror` vs `anyhow` vs manual `impl Error`, whether it uses `#[derive(Debug)]` everywhere or selectively, whether tests use `assert_eq!` vs `assert_matches!`. These tiny signals are what make AI code feel alien or native.

---

### Feature 4: Swarm Hive Mind — Inter-Agent Communication
**Wow Factor:** 9/10
**The Dream:** Agents can talk to each other during execution. When executor-2 discovers that executor-1's task created a new public API it needs, it can query a shared "hive mind" channel. When executor-3 hits a blocker, it broadcasts a help request. A lightweight message bus lets parallel agents share context, negotiate file ownership, and avoid conflicts *before* they happen — turning independent workers into a collaborative team.
**Why It's Remarkable:** Current swarm orchestrators treat agents as isolated processes. But real engineering teams communicate constantly. The hive mind bridges the gap between "parallel execution" and "collaborative intelligence."
**How To Build It:**
1. Create `src/hivemind.rs` with a `HiveMind` struct backed by `Arc<Mutex<Vec<HiveMessage>>>` — each message has sender_id, message_type (Announcement, Query, FileOwnership, Blocker), content, timestamp
2. Implement a `HiveMindFile` that serializes to `.subspace/hivemind.json` — agents running in worktrees can read this shared file
3. Inject a "Hive Mind Protocol" section into executor prompts: instruct agents to write `.subspace/hivemind-outbox/{agent-id}.json` with messages, and check `.subspace/hivemind.json` at tool-call boundaries
4. In `swarm.rs`, add a background tokio task that polls outbox files every 2s, merges them into the shared hivemind.json, and handles FileOwnership claims (first-come-first-served)
5. TUI: add a "Hive" panel (toggled with `h` key) showing the message stream with agent avatars
6. File ownership prevents conflicts: if executor-1 claims `src/auth.rs`, executor-2's prompt gets updated to avoid that file
**Files To Create/Modify:**
- `src/hivemind.rs` (new — message bus, file ownership tracker, outbox poller)
- `src/swarm.rs` (spawn hivemind background task, inject protocol into prompts)
- `src/dream.rs` (same injection for dream builders)
- `src/tui.rs` (hive message panel)
**Creative Notes:** The killer feature is *proactive conflict prevention*. Instead of merging and fixing, agents pre-negotiate. The ownership system could even implement a "draft pick" — planner assigns file ownership at planning time, agents must trade to modify each other's files.

---

### Feature 5: Dream Chain Reactions — Cascading Innovation
**Wow Factor:** 8/10
**The Dream:** After dream mode builds a feature, Subspace automatically re-analyzes the project to discover *second-order features* unlocked by the new capability. Built a replay system? Now dream "replay diffing." Built a DNA extractor? Now dream "style drift alerts." Each innovation triggers a chain reaction of derivative ideas, forming an ever-expanding feature tree that the project grows along organically.
**Why It's Remarkable:** Current dream mode is a flat loop — dream, build, dream, build. Chain reactions add *depth*. The project evolves not randomly but along branching paths of possibility, where each feature naturally suggests the next. It's evolution, not just invention.
**How To Build It:**
1. Create `src/dream_chain.rs` with a `DreamGraph` struct: nodes are built features (from DREAMS.md), edges are "enabled-by" relationships
2. After each dream cycle completes in `dream.rs`, inject the *just-built features* into the next dreamer prompt with the directive: "These features were just built. What second-order features do they now make possible that weren't before?"
3. Parse dreamer output for `<chain_parent>Feature Name</chain_parent>` tags linking new dreams to their parents
4. Serialize the graph to `.subspace/dream-graph.json`
5. Add `subspace dream --graph` that renders the dream tree as an ASCII art dependency graph in the terminal
6. TUI: show the dream tree as a visual node graph in a dedicated panel, with built features as green nodes and pending dreams as yellow
7. Add a `--depth N` flag to limit chain reaction depth (default: 3 levels deep)
**Files To Create/Modify:**
- `src/dream_chain.rs` (new — graph structure, chain prompt injection, ASCII renderer)
- `src/dream.rs` (inject chain context, parse parent tags, update graph after each cycle)
- `src/cli.rs` (add `--graph` and `--depth` flags to dream subcommand)
- `src/tui.rs` (dream tree visualization widget)
**Creative Notes:** The graph becomes a *project roadmap that writes itself*. Export it as a Mermaid diagram for docs. The depth limit prevents runaway chains, but a `--infinite` mode could let the project evolve for hours unattended, following its own branching logic.

---

### Feature 6: Phantom Agents — Dry-Run Simulation
**Wow Factor:** 7/10
**The Dream:** Before committing to a full swarm run, spawn "phantom" agents that simulate execution without writing any files. They read the codebase, plan their changes, and report *what they would do* — files they'd create, functions they'd modify, estimated diff size, potential conflicts with other phantoms — all in seconds instead of minutes. It's a pre-flight check for your swarm.
**Why It's Remarkable:** Swarm runs are expensive (time, API calls, potential merge conflicts). Phantom mode lets you preview the blast radius before pulling the trigger. It's the `--dry-run` that every DevOps tool should have but AI orchestrators never do.
**How To Build It:**
1. Create `src/phantom.rs` with a `PhantomRun` struct: takes a plan's tasks, spawns agents with a modified prompt that says "Describe exactly what you would do, which files you'd modify, and what the changes would look like — but do NOT make any changes"
2. The phantom prompt instructs agents to output `<phantom_plan>` XML with: files_to_create, files_to_modify, functions_to_change, estimated_lines_changed, dependencies_needed
3. Parse all phantom outputs, cross-reference for conflicts, generate a "Simulation Report"
4. Add `subspace run --phantom` and `subspace dream --phantom` flags
5. TUI: show phantom results as a preview panel with green (safe), yellow (potential conflict), red (definite conflict) indicators per file
6. After review, user can confirm to proceed with actual execution using the same plan
**Files To Create/Modify:**
- `src/phantom.rs` (new — simulation orchestrator, phantom prompt builder, report generator, conflict cross-reference)
- `src/cli.rs` (add `--phantom` flag to run and dream)
- `src/swarm.rs` (call phantom mode before real execution when flag set)
- `src/tui.rs` (simulation preview widget)
**Creative Notes:** Phantom agents use Haiku (fast + cheap) instead of Sonnet. The simulation report could include a "confidence score" — how certain the phantom is about its plan. Low confidence = maybe split the task further. This turns the swarm from "fire and pray" into "plan, preview, execute."

</dream_features>