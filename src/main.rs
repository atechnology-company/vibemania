mod cli;
mod tmux;
mod agent;
mod planner;
mod executor;
mod swarm;
mod project;
mod conflict;
mod config;
mod worktree;
mod merger;
mod acp;

use anyhow::Result;
use clap::Parser;
use colored::*;
use cli::{Cli, Commands, SwarmCommands};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            project::init(&dir)?;
        }
        Commands::Plan { goal, project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load_or_init(&dir)?;
            let prompt = planner::build_prompt(&proj, &goal, 1);
            let result = acp::run_prompt(&dir, &prompt).await?;
            println!("{}", result.text);
        }
        Commands::Run { goal, project_dir, max_iter, parallel } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load_or_init(&dir)?;
            swarm::run_loop(&proj, &goal, max_iter, parallel).await?;
        }
        Commands::Swarm { command } => match command {
            SwarmCommands::Launch { goal, project_dir, agents } => {
                let dir = project::resolve_dir(project_dir)?;
                let proj = project::load_or_init(&dir)?;
                swarm::launch(&proj, &goal, agents).await?;
            }
            SwarmCommands::Status => swarm::status().await?,
            SwarmCommands::Logs { agent_id } => swarm::logs(&agent_id).await?,
            SwarmCommands::Merge { project_dir } => {
                let dir = project::resolve_dir(project_dir)?;
                let proj = project::load_or_init(&dir)?;
                let state = config::load_state(&dir)?;
                if let Some(state) = state {
                    let base = worktree::current_branch(&dir)?;
                    let completed: Vec<String> = state.agents.iter()
                        .filter(|a| a.role == config::AgentRole::Executor && a.status == config::AgentStatus::Completed)
                        .map(|a| a.id.clone()).collect();
                    if completed.is_empty() { println!("No completed agents."); return Ok(()); }

                    let subspace_dir = dir.join(".subspace");
                    let mut details = String::new();
                    for id in &completed {
                        let d = worktree::diff_from_base(&dir, id, &base).unwrap_or_default();
                        let f = worktree::full_diff(&dir, id, &base).unwrap_or_default();
                        details.push_str(&format!("### subspace/{}\n```\n{}\n```\n```diff\n{}\n```\n---\n", id, d.trim(), f.trim()));
                    }
                    let prompt = format!(
                        "Merge these branches into `{}`: {}\n\n{}\n\nResolve conflicts. Run: {}. Commit.",
                        base, completed.iter().map(|id| format!("subspace/{}", id)).collect::<Vec<_>>().join(", "),
                        details, project::detect_stack(&dir).as_str(),
                    );
                    println!("Merging {} branch(es)...", completed.len());
                    let r = acp::run_prompt(&dir, &prompt).await?;
                    std::fs::write(subspace_dir.join("merger-output.md"), &r.text)?;
                    println!("{} Done", "✓".green());
                } else { println!("No active swarm."); }
            }
            SwarmCommands::Clean { project_dir } => {
                let dir = project::resolve_dir(project_dir)?;
                swarm::clean(&dir)?;
            }
        },
        Commands::Status { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load_or_init(&dir)?;
            project::show_status(&proj)?;
        }
        Commands::Conflicts { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load_or_init(&dir)?;
            conflict::show(&proj)?;
        }
    }

    Ok(())
}
