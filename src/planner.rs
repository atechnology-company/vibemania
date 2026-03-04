use anyhow::Result;
use regex::Regex;

use crate::agent;
use crate::config::AgentRole;
use crate::project::Project;

const PLANNER_TEMPLATE: &str = include_str!("../prompts/planner.md");

#[derive(Debug, Clone)]
pub struct PlannedTask {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub files: Vec<String>,
}

/// Build the planner prompt with project context
pub fn build_prompt(proj: &Project, iteration: u32) -> String {
    let prompt = PLANNER_TEMPLATE
        .replace("{{ITERATION}}", &iteration.to_string())
        .replace("{{STACK}}", &proj.stack);

    format!(
        "{}\n\n## Current Goals\n\n{}\n\n## Progress So Far\n\n{}",
        prompt, proj.goals, proj.progress
    )
}

/// Write prompt to a temp file and return path
pub fn write_prompt(proj: &Project, iteration: u32) -> Result<String> {
    let subspace_dir = proj.dir.join(".subspace");
    std::fs::create_dir_all(&subspace_dir)?;
    let path = subspace_dir.join("planner-prompt.md");
    let prompt = build_prompt(proj, iteration);
    std::fs::write(&path, &prompt)?;
    Ok(path.to_string_lossy().to_string())
}

/// Parse planner output into tasks
pub fn parse_tasks(output: &str) -> Vec<PlannedTask> {
    // Check for multi-task format
    if output.contains("<vibemania_tasks") || output.contains("<subspace_tasks") {
        parse_multi_tasks(output)
    } else {
        // Single task
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
    output.contains("<vibemania>COMPLETE</vibemania>")
        || output.contains("<subspace>COMPLETE</subspace>")
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
            tasks.push(PlannedTask {
                id,
                title,
                content: full,
                files,
            });
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

/// Run the planner and return its output
pub async fn run_planner(proj: &Project, tool: &str) -> Result<String> {
    let prompt_file = write_prompt(proj, 1)?;
    let session = crate::project::session_name(&proj.dir);

    println!("Running planner...");

    let _agent_state = agent::spawn(
        &session,
        "planner",
        AgentRole::Planner,
        tool,
        &prompt_file,
        &proj.dir.to_string_lossy(),
        None,
    )?;

    // Wait for planner to finish
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        if agent::is_done(&session, "planner")? || !agent::is_running(&session, "planner") {
            break;
        }
    }

    let output = agent::get_output(&session, "planner", 500)?;
    Ok(output)
}
