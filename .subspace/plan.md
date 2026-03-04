Now let me run the audit to check compilation and test status.Now let me check the tokio-util usage to evaluate that task.Audit complete. Summary:
- **cargo check**: passes (45 dead_code warnings from unwired modules, expected)
- **cargo clippy**: 61 warnings (45 dead_code + 16 clippy lints including `empty_line_after_outer_attr`, `manual_strip`, `collapsible_if`, `print_literal`, `filter_next`, `collapsible_else_if`)
- **cargo test**: no tests exist yet (linker not available in this env, but compiles fine)
- **tokio-util**: still actively used in `src/acp.rs` for `compat_write()`/`compat()` — task should be marked skipped

The clippy warnings task was marked [x] but there are still new clippy lints. Let me also note that the existing clippy fixes seem to be in uncommitted changes based on git status.

**Parallelizability analysis** of the 3 test tasks:
- `subspace_file.rs` tests — only touches `src/subspace_file.rs`
- `planner.rs` tests — only touches `src/planner.rs`
- `project.rs` tests — only touches `src/project.rs`

All three are perfectly independent. Now let me update subspace.md and output tasks.<subspace_tasks max_parallel="3">

### Task 1: Add unit tests for subspace_file parsing
#### Priority: 🟡
#### What
Add a `#[cfg(test)] mod tests` block at the bottom of `src/subspace_file.rs` with the following tests:

1. **`test_parse_empty`** — `SubspaceFile::parse("")` returns a default struct with empty fields
2. **`test_parse_project_section`** — Parse a markdown string with `## Project` containing Name, Description, Completion and verify all fields are extracted correctly
3. **`test_parse_roadmap`** — Parse markdown with `## Roadmap` containing `- [x] Done item` and `- [ ] Todo item`, verify `roadmap` vec has correct `done` flags and descriptions
4. **`test_parse_tasks_section`** — Parse markdown with `## Tasks` containing:
   - `### [x] 🔴 Critical Task` with description and file list
   - `### [ ] 🟡 Medium Task` with description
   - Verify task count, statuses, priorities, titles, files, descriptions
5. **`test_parse_task_statuses`** — Test all status markers: `[x]` → Done, `[~]` → InProgress, `[-]` → Skipped, `[ ]` → Todo
6. **`test_parse_completed_section`** — Parse `## Completed` with bullet items
7. **`test_parse_notes_section`** — Parse `## Notes` with multiline text
8. **`test_pending_tasks`** — Create SubspaceFile with mixed Done/Todo/InProgress tasks, verify `pending_tasks()` returns only Todo+InProgress sorted by priority (Critical first)
9. **`test_to_markdown_roundtrip`** — Build a SubspaceFile programmatically, call `to_markdown()`, then `parse()` the result, verify fields match the original
10. **`test_parse_task_header`** — Test the `parse_task_header` function (make it `pub(crate)` if needed) with various inputs including emoji prefixes

Each test should be self-contained with inline markdown strings. No file I/O needed.

#### Files Affected
- src/subspace_file.rs

#### Quality Checks
cargo check
cargo test --lib -- subspace_file (once linker is available)

### Task 2: Add unit tests for planner task parsing
#### Priority: 🟡
#### What
Add a `#[cfg(test)] mod tests` block at the bottom of `src/planner.rs` with the following tests:

1. **`test_is_complete_subspace`** — `is_complete("<subspace>COMPLETE</subspace>")` returns true
2. **`test_is_complete_vibemania`** — `is_complete("<vibemania>COMPLETE</vibemania>")` returns true
3. **`test_is_complete_false`** — `is_complete("some other text")` returns false
4. **`test_parse_single_task`** — Input without `<subspace_tasks>` returns a single PlannedTask with extracted title and files
5. **`test_parse_multi_tasks`** — Input with `<subspace_tasks>` block containing `### Task 1: Title A` and `### Task 2: Title B` returns 2 PlannedTasks with correct ids, titles
6. **`test_extract_files`** — Input with `#### Files Affected\n- src/foo.rs\n- src/bar.rs` extracts both files correctly
7. **`test_extract_files_empty`** — Input with no "Files Affected" section returns empty vec
8. **`test_extract_title`** — Input with `### Some Title` (no "Task N:" prefix) extracts "Some Title"
9. **`test_extract_title_fallback`** — Input with no `###` header returns "Unnamed task"
10. **`test_parse_tasks_with_quality_checks`** — Full realistic planner output with tasks, files, and quality checks sections parses correctly

Note: `extract_title` and `extract_files` are private. Test them indirectly via `parse_tasks`. Alternatively, make them `pub(crate)` for direct testing.

#### Files Affected
- src/planner.rs

#### Quality Checks
cargo check
cargo test --lib -- planner (once linker is available)

### Task 3: Add unit tests for project stack detection
#### Priority: 🟡
#### What
Add a `#[cfg(test)] mod tests` block at the bottom of `src/project.rs` with the following tests:

1. **`test_detect_stack_rust`** — Create a temp dir with `Cargo.toml`, verify `detect_stack()` returns "rust"
2. **`test_detect_stack_node`** — Create temp dir with `package.json`, verify returns "node"
3. **`test_detect_stack_typescript`** — Create temp dir with `package.json` + `tsconfig.json`, verify returns "node,typescript"
4. **`test_detect_stack_python_requirements`** — Create temp dir with `requirements.txt`, verify returns "python"
5. **`test_detect_stack_python_pyproject`** — Create temp dir with `pyproject.toml`, verify returns "python" (not duplicate)
6. **`test_detect_stack_go`** — Create temp dir with `go.mod`, verify returns "go"
7. **`test_detect_stack_multiple`** — Create temp dir with `Cargo.toml` + `Dockerfile`, verify returns "rust,docker"
8. **`test_detect_stack_unknown`** — Empty temp dir returns "unknown"
9. **`test_detect_stack_nextjs`** — Create temp dir with `package.json` + `next.config.js`, verify returns "node,nextjs"
10. **`test_session_name`** — Verify `session_name(Path::new("/foo/my-project"))` returns "subspace-my-project"

Use `tempfile::tempdir()` or `std::env::temp_dir()` with unique subdirs. Create empty marker files (just `std::fs::write(path, "")`). Clean up after.

Note: The `detect_stack` function checks file existence deterministically in order of the `checks` array, and deduplicates by value (e.g. two python markers → one "python"). Tests should verify this dedup behavior.

#### Files Affected
- src/project.rs

#### Quality Checks
cargo check
cargo test --lib -- project (once linker is available)

</subspace_tasks>**Plan summary:**

- **subspace.md updated** — marked tokio-util task as `[-]` skipped (still needed), marked 3 test tasks as `[~]` in-progress, noted clippy regressions
- **3 parallel tasks output** — all test additions to independent files (`subspace_file.rs`, `planner.rs`, `project.rs`)
- **Deferred to next iteration**: clippy fixes (touches many files, conflicts with test tasks), tmux wiring, CI workflow