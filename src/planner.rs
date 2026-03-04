use anyhow::Result;
use regex::Regex;

use crate::project::Project;

/// Build the planner prompt — planner manages progress tracking
pub fn build_prompt(proj: &Project, goal: &str, iteration: u32) -> String {
    let progress = if proj.progress.is_empty() || proj.progress == "No progress yet." {
        "This is the first iteration. No progress yet.".to_string()
    } else {
        proj.progress.clone()
    };

    format!(
        r#"# Subspace Planner Agent

You are the PLANNING phase of Subspace, an AI coding agent swarm orchestrator.
This is iteration {iteration}. Detected project stack: {stack}.

## The Goal

{goal}

## Progress So Far

{progress}

## Your Job

1. Read the project's current state (check files, git log, etc.)
2. Compare against the goal
3. If the goal is FULLY achieved: output `<subspace>COMPLETE</subspace>` and nothing else
4. Otherwise: output specific, actionable tasks for executor agents

## First Iteration Tasks

If this is iteration 1, also:
- Create/update `progress.md` with the goal and initial state
- Set up any scaffolding needed

## Task Output Format

For a SINGLE task:
### Task Title
[one-line summary]
### Why This Is Next
[brief reasoning]
### Detailed Instructions
[step-by-step for an AI developer]
### Files Affected
- path/to/file1
- path/to/file2
### Quality Checks
[commands to verify]

For MULTIPLE parallel tasks:
<subspace_tasks max_parallel="3">

### Task 1: Title
#### Why This Is Next
...
#### Detailed Instructions
...
#### Files Affected
- file1
- file2
#### Quality Checks
...

### Task 2: Title
...

</subspace_tasks>

## Rules

- Plan tasks that can run IN PARALLEL (no shared files between tasks)
- Be SPECIFIC: "Add auth middleware to src/auth.rs" not "improve auth"
- Each task must be completable by one agent in isolation
- List ALL files each task will touch (for conflict avoidance)
- If tasks MUST be sequential, output only ONE task"#,
        iteration = iteration,
        stack = proj.stack,
        goal = goal,
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

/// Parse planner output into tasks
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

/// Check if planner says we're done
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
        if trimmed.starts_with("## ") {
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
                let file = trimmed
                    .trim_start_matches("- ")
                    .trim_start_matches("* ")
                    .trim()
                    .to_string();
                if !file.is_empty() && (file.contains('/') || file.contains('.')) {
                    files.push(file);
                }
            } else if trimmed.starts_with('#') || trimmed.is_empty() {
                if !files.is_empty() {
                    in_files_section = false;
                }
            }
        }
    }
    files
}
