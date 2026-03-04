use regex::Regex;

use crate::project::Project;

/// Build the planner prompt — autonomous discovery + audit + roadmap
pub fn build_prompt(proj: &Project, goal: Option<&str>, iteration: u32) -> String {
    let progress = if proj.progress.is_empty() || proj.progress == "No progress yet." {
        "First iteration — no prior progress.".to_string()
    } else {
        proj.progress.clone()
    };

    let goal_section = match goal {
        Some(g) => format!("## Explicit Goal\n\n{}\n\nUse this as your PRIMARY focus, but still audit the full project.", g),
        None => "## Goal: Autonomous Completion\n\nNo explicit goal given. Your job is to take this project from its current state to 95% complete.".to_string(),
    };

    format!(
        r#"# Subspace Planner — Autonomous Project Engine

You are the brain of Subspace, an AI swarm that takes projects from 0% to 95%.
This is iteration {iteration}. Stack: {stack}.

{goal_section}

## Phase 1: Discovery (DO THIS FIRST)

Read and understand the project by checking these files (if they exist):
- `CLAUDE.md` / `.claude/instructions.md` — Claude Code agent instructions
- `.copilot-instructions.md` / `.github/copilot-instructions.md` — Copilot context
- `GEMINI.md` / `.gemini/instructions.md` — Gemini context
- `README.md` — Project overview, setup, architecture
- `TODO.md` / `TODO` — Existing task lists
- `ROADMAP.md` — Project roadmap
- `CHANGELOG.md` — What's been done
- `package.json` / `Cargo.toml` / `pyproject.toml` — Dependencies, scripts
- `goals.md` — Prior Subspace goals (if any)
- `.github/ISSUES_TEMPLATE/` — What issues look like
- Source code structure (`src/`, `lib/`, `app/`, etc.)
- Test directories (`tests/`, `__tests__/`, `spec/`)
- Config files (CI, linting, formatting)

Synthesize ALL of this into your understanding of the project.

## Phase 2: Audit

Analyze the project's current state:
1. **Completeness** — What features exist vs what's described/planned?
2. **Quality** — Do tests pass? Are there linting errors? Type errors?
3. **Gaps** — Missing tests, missing error handling, missing docs?
4. **Broken things** — Compile errors, failing tests, dead code?
5. **Architecture** — Is the structure sound? Any anti-patterns?
6. **Dependencies** — Outdated? Vulnerable? Missing?
7. **CI/CD** — Is it set up? Does it work?
8. **Documentation** — README up to date? API docs? Comments?

Run actual commands to verify:
- Build/compile the project
- Run the test suite
- Run linters
- Check for type errors

## Phase 3: Roadmap & Task Planning

Based on your discovery and audit:

1. Estimate current completion percentage
2. Create a prioritized roadmap of what's needed to reach 95%
3. Break the next chunk into PARALLEL tasks for executor agents

Priority order:
1. 🔴 Fix broken things (won't compile, tests fail)
2. 🟠 Complete core features (what the project is supposed to do)
3. 🟡 Add missing tests & error handling
4. 🟢 Documentation, polish, CI/CD
5. 🔵 Nice-to-haves, optimization

## Progress So Far

{progress}

## Output Format

### Project Assessment
- **Name:** [project name]
- **Description:** [what it does]
- **Current State:** [% complete, key observations]
- **Critical Issues:** [anything broken]

### Roadmap to 95%
1. [Phase 1 — what, why, est. effort]
2. [Phase 2 — ...]
3. [Phase 3 — ...]

### This Iteration's Tasks

If DONE: output `<subspace>COMPLETE</subspace>`

If tasks remain, output them:

<subspace_tasks max_parallel="N">

### Task 1: [Title]
#### Priority: 🔴/🟠/🟡/🟢/🔵
#### What
[specific instructions]
#### Files Affected
- path/to/file
#### Quality Checks
[commands to run]

### Task 2: [Title]
...

</subspace_tasks>

### Progress Update

Write this to `progress.md`:
```
## Iteration {iteration}
Date: [today]
Assessment: [current state]
Completed: [what was done]
Next: [what's planned]
```

## Rules

- ALWAYS do discovery first — read the project's own docs
- Respect existing agent instructions (CLAUDE.md etc.)
- Tasks must be PARALLELIZABLE (no shared files)
- Be SPECIFIC — "Add error handling to src/api/users.rs lines 45-80" not "improve error handling"
- Each task must list ALL files it touches
- Don't rewrite working code — fix and extend
- If the project is already at 95%+, say COMPLETE"#,
        iteration = iteration,
        stack = proj.stack,
        goal_section = goal_section,
        progress = progress,
    )
}

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
        let trimmed = line.trim();
        if trimmed.starts_with("### ") && !trimmed.contains("Task ") {
            return trimmed.trim_start_matches("### ").to_string();
        }
        if trimmed.starts_with("## ") && !trimmed.contains("Phase") && !trimmed.contains("Progress") && !trimmed.contains("Rules") {
            return trimmed.trim_start_matches("## ").to_string();
        }
    }
    "Unnamed task".to_string()
}

fn extract_files(content: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut in_files_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Files Affected") || trimmed.contains("Files to Modify") {
            in_files_section = true;
            continue;
        }
        if in_files_section {
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let file = trimmed.trim_start_matches("- ").trim_start_matches("* ").trim().to_string();
                if !file.is_empty() && (file.contains('/') || file.contains('.')) {
                    files.push(file);
                }
            } else if trimmed.starts_with('#') || trimmed.is_empty() {
                if !files.is_empty() { in_files_section = false; }
            }
        }
    }
    files
}
