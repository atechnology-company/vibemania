use regex::Regex;

use crate::project::Project;
use crate::subspace_file::SubspaceFile;

/// Build the planner prompt.
/// 
/// Modes:
/// - **Tasks mode**: subspace.md exists with tasks → work through them
/// - **Autonomous mode**: no tasks → discover, audit, build roadmap, generate tasks
pub fn build_prompt(proj: &Project, goal: Option<&str>, iteration: u32) -> String {
    let sf = SubspaceFile::load_or_default(&proj.dir).unwrap_or_default();
    let has_tasks = sf.has_tasks();
    let pending = sf.pending_tasks();

    let progress = if proj.progress.is_empty() || proj.progress == "No progress yet." {
        "First iteration — no prior progress.".to_string()
    } else {
        proj.progress.clone()
    };

    let subspace_context = if has_tasks {
        format!(
            "## subspace.md (Existing Project State)\n\n\
             - Completion: {}%\n\
             - Pending tasks: {}\n\
             - Completed: {}\n\n\
             ### Pending Tasks\n{}\n\n\
             ### Completed\n{}",
            sf.completion_pct,
            pending.len(),
            sf.completed.len(),
            pending.iter().map(|t| format!("- [ ] **{}** — {}", t.title, t.description.lines().next().unwrap_or(""))).collect::<Vec<_>>().join("\n"),
            sf.completed.iter().map(|c| format!("- [x] {}", c)).collect::<Vec<_>>().join("\n"),
        )
    } else {
        "No subspace.md yet — create one as part of your output.".to_string()
    };

    let mode_instructions = if has_tasks && !pending.is_empty() {
        tasks_mode_instructions(&sf, iteration)
    } else if has_tasks && pending.is_empty() {
        "All tasks in subspace.md are complete. Do a final audit and either mark COMPLETE or add new tasks.".to_string()
    } else {
        autonomous_mode_instructions(goal, iteration)
    };

    format!(
        r#"# Subspace Planner — Autonomous Project Completion Engine

Stack: **{stack}** | Iteration: **{iteration}**

{subspace_context}

---

{mode_instructions}

---

## Previous Progress

{progress}

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

<subspace_tasks max_parallel="{parallel}">

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
- Don't redo completed work"#,
        stack = proj.stack,
        iteration = iteration,
        subspace_context = subspace_context,
        mode_instructions = mode_instructions,
        progress = progress,
        parallel = 3,
    )
}

fn autonomous_mode_instructions(goal: Option<&str>, _iteration: u32) -> String {
    let goal_line = match goal {
        Some(g) => format!("**Focus goal:** {}\n\nUse this as your primary lens, but still do full discovery and audit.", g),
        None => "**Mode:** Fully autonomous — discover what needs to be done and do it.".to_string(),
    };

    format!(r#"## Mode: Autonomous Discovery

{goal_line}

### Step 1: Discovery

Read everything you can find about this project:

| File | Purpose |
|------|---------|
| `CLAUDE.md` / `.claude/instructions.md` | Claude Code agent context |
| `.copilot-instructions.md` / `.github/copilot-instructions.md` | Copilot context |
| `GEMINI.md` / `.gemini/instructions.md` | Gemini context |
| `README.md` | Overview, setup, architecture |
| `TODO.md` / `TODO` | Existing task lists |
| `ROADMAP.md` | Planned work |
| `CHANGELOG.md` | What's been shipped |
| `package.json` / `Cargo.toml` / `pyproject.toml` | Deps, scripts, entry points |
| `src/` / `lib/` / `app/` | Source structure |
| `tests/` / `__tests__/` / `spec/` | Test coverage |
| `.github/workflows/` | CI/CD setup |

### Step 2: Audit

Run commands to find actual problems:
- Build/compile the project
- Run the test suite
- Run linters/type checkers
- Check for TODO/FIXME comments in code
- Check for missing error handling
- Identify dead code or unused dependencies

### Step 3: Create subspace.md

Based on your discovery and audit:
1. Estimate completion percentage
2. Write a roadmap (phases to reach 95%)
3. Create a prioritized task list
4. Pick the top N parallelizable tasks for this iteration

Priority order:
1. 🔴 **Critical** — won't compile, tests fail, crashes
2. 🟠 **High** — core features incomplete or missing  
3. 🟡 **Medium** — missing tests, poor error handling
4. 🟢 **Low** — docs, comments, cleanup
5. 🔵 **Nice** — optimization, nice-to-haves"#,
        goal_line = goal_line,
    )
}

fn tasks_mode_instructions(sf: &SubspaceFile, _iteration: u32) -> String {
    let pending = sf.pending_tasks();
    let next_tasks: Vec<String> = pending.iter().take(5)
        .map(|t| {
            let p = match t.priority {
                crate::subspace_file::TaskPriority::Critical => "🔴",
                crate::subspace_file::TaskPriority::High => "🟠",
                crate::subspace_file::TaskPriority::Medium => "🟡",
                crate::subspace_file::TaskPriority::Low => "🟢",
                crate::subspace_file::TaskPriority::Nice => "🔵",
            };
            format!("- {} **{}** — {}", p, t.title, t.description.lines().next().unwrap_or(""))
        })
        .collect();

    format!(r#"## Mode: Tasks

subspace.md has **{}** pending tasks. Work through them in priority order.

### Next Up
{}

### Instructions

1. **Read the current codebase** to understand the context for each task
2. **Pick the top parallelizable tasks** (no shared files between them)
3. **Run a quick audit** — do tests pass? Does it compile? Any blockers?
4. **Output tasks** for executor agents (use the exact task titles from subspace.md)
5. **Update subspace.md** — mark in-progress tasks as `[~]`, remove blockers

Do NOT invent new tasks unless the audit reveals critical issues not in the list.
If a task in subspace.md is already done (check the codebase), mark it `[x]`."#,
        sf.pending_tasks().len(),
        next_tasks.join("\n"),
    )
}

// ─── Task parsing (unchanged) ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlannedTask {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub files: Vec<String>,
}

pub fn parse_tasks(output: &str) -> Vec<PlannedTask> {
    if output.contains("<subspace_tasks") || output.contains("<vibemania_tasks") {
        parse_multi_tasks(output)
    } else {
        vec![PlannedTask {
            id: 1,
            title: extract_title(output),
            content: output.to_string(),
            files: extract_files(output),
        }]
    }
}

pub fn is_complete(output: &str) -> bool {
    output.contains("<subspace>COMPLETE</subspace>")
        || output.contains("<vibemania>COMPLETE</vibemania>")
}

fn parse_multi_tasks(output: &str) -> Vec<PlannedTask> {
    let mut tasks = Vec::new();
    let re = Regex::new(r"###\s+Task\s+(\d+):\s*(.+)").unwrap();
    let sections: Vec<&str> = output.split("### Task ").collect();

    for section in sections.iter().skip(1) {
        let full = format!("### Task {}", section);
        if let Some(caps) = re.captures(&full) {
            let id: u32 = caps[1].parse().unwrap_or(tasks.len() as u32 + 1);
            let title = caps[2].trim().to_string();
            let files = extract_files(&full);
            tasks.push(PlannedTask { id, title, content: full, files });
        }
    }

    if tasks.is_empty() {
        vec![PlannedTask {
            id: 1,
            title: extract_title(output),
            content: output.to_string(),
            files: extract_files(output),
        }]
    } else {
        tasks
    }
}

fn extract_title(output: &str) -> String {
    for line in output.lines() {
        let t = line.trim();
        if t.starts_with("### ") && !t.contains("Task ") {
            return t.trim_start_matches("### ").to_string();
        }
    }
    "Unnamed task".to_string()
}

fn extract_files(content: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut in_files = false;
    for line in content.lines() {
        let t = line.trim();
        if t.contains("Files Affected") || t.contains("Files to Modify") {
            in_files = true; continue;
        }
        if in_files {
            if t.starts_with("- ") || t.starts_with("* ") {
                let f = t.trim_start_matches("- ").trim_start_matches("* ").trim().to_string();
                if !f.is_empty() && (f.contains('/') || f.contains('.')) { files.push(f); }
            } else if t.starts_with('#') || t.is_empty() {
                if !files.is_empty() { in_files = false; }
            }
        }
    }
    files
}
