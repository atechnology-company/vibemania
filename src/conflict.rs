use anyhow::Result;
use colored::*;
use std::collections::HashMap;

use crate::config::{self, FileConflict, SwarmState};
use crate::project::Project;

/// Detect file conflicts between agents
pub fn detect(state: &SwarmState) -> Vec<FileConflict> {
    let mut file_agents: HashMap<String, Vec<String>> = HashMap::new();

    for agent in &state.agents {
        for file in &agent.files_modified {
            file_agents
                .entry(file.clone())
                .or_default()
                .push(agent.id.clone());
        }
        // Also check task-declared files
        if let Some(task) = &agent.task {
            for file in &task.files {
                file_agents
                    .entry(file.clone())
                    .or_default()
                    .push(agent.id.clone());
            }
        }
    }

    file_agents
        .into_iter()
        .filter(|(_, agents)| agents.len() > 1)
        .map(|(file, agents)| FileConflict { file, agents })
        .collect()
}

/// Show file conflicts for a project
pub fn show(proj: &Project) -> Result<()> {
    let state = config::load_state(&proj.dir)?;
    match state {
        None => {
            println!("{}", "No active swarm state found.".dimmed());
        }
        Some(state) => {
            let conflicts = detect(&state);
            if conflicts.is_empty() {
                println!("{}", "No file conflicts detected. ✓".green());
            } else {
                println!(
                    "{} {} file conflict(s) detected:",
                    "⚠".red(),
                    conflicts.len()
                );
                println!();
                for c in &conflicts {
                    println!("  {} {}", "→".red(), c.file.yellow());
                    for agent in &c.agents {
                        println!("    modified by: {}", agent.cyan());
                    }
                }
            }
        }
    }
    Ok(())
}
