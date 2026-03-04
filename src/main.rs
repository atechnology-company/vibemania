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
mod subspace_file;

use anyhow::Result;
use clap::Parser;
use colored::*;
use cli::{Cli, Commands, SwarmCommands};
use subspace_file::{SubspaceFile, Task, TaskPriority, TaskStatus};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            project::init(&dir)?;
            // Create empty subspace.md if it doesn't exist
            let sf_path = SubspaceFile::path(&dir);
            if !sf_path.exists() {
                let sf = SubspaceFile::default();
                sf.save(&dir)?;
                println!("{} Created subspace.md", "✓".green());
            }
        }
        Commands::Plan { goal, project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load_or_init(&dir)?;
            let prompt = planner::build_prompt(&proj, goal.as_deref(), 1);
            let result = acp::run_prompt(&dir, &prompt).await?;
            println!("{}", result.text);
        }
        Commands::Run { goal, project_dir, max_iter, parallel } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load_or_init(&dir)?;
            swarm::run_loop(&proj, goal.as_deref(), max_iter, parallel).await?;
        }
        Commands::Swarm { command } => match command {
            SwarmCommands::Launch { goal, project_dir, agents } => {
                let dir = project::resolve_dir(project_dir)?;
                let proj = project::load_or_init(&dir)?;
                swarm::launch(&proj, goal.as_deref(), agents).await?;
            }
            SwarmCommands::Status => swarm::status().await?,
            SwarmCommands::Logs { agent_id } => swarm::logs(&agent_id).await?,
            SwarmCommands::Merge { project_dir } => {
                let dir = project::resolve_dir(project_dir)?;
                swarm::merge(&dir).await?;
            }
            SwarmCommands::Clean { project_dir } => {
                let dir = project::resolve_dir(project_dir)?;
                swarm::clean(&dir)?;
            }
        },
        Commands::Tasks { project_dir, add, priority } => {
            let dir = project::resolve_dir(project_dir)?;
            let mut sf = SubspaceFile::load_or_default(&dir)?;

            if let Some(title) = add {
                let prio = match priority.to_lowercase().as_str() {
                    "critical" => TaskPriority::Critical,
                    "high" => TaskPriority::High,
                    "low" => TaskPriority::Low,
                    "nice" => TaskPriority::Nice,
                    _ => TaskPriority::Medium,
                };
                let id = sf.tasks.len() as u32 + 1;
                sf.tasks.push(Task {
                    id,
                    title: title.clone(),
                    priority: prio,
                    status: TaskStatus::Todo,
                    files: Vec::new(),
                    description: String::new(),
                });
                sf.save(&dir)?;
                println!("{} Added task: {}", "✓".green(), title);
            } else {
                // Show tasks
                let sf_path = SubspaceFile::path(&dir);
                if !sf_path.exists() {
                    println!("{}", "No subspace.md found. Run `subspace init` or `subspace run` first.".dimmed());
                    return Ok(());
                }
                println!("{}", "═══ subspace.md Tasks ═══".bold());
                println!("Project: {} | {}% complete", sf.project_name.cyan(), sf.completion_pct);
                println!();

                let pending = sf.pending_tasks();
                if pending.is_empty() {
                    println!("{}", "No pending tasks.".dimmed());
                } else {
                    println!("Pending ({}):", pending.len());
                    for t in &pending {
                        let p = match t.priority {
                            TaskPriority::Critical => "🔴",
                            TaskPriority::High => "🟠",
                            TaskPriority::Medium => "🟡",
                            TaskPriority::Low => "🟢",
                            TaskPriority::Nice => "🔵",
                        };
                        let s = if t.status == TaskStatus::InProgress { "~" } else { " " };
                        println!("  [{}] {} {}", s, p, t.title.bold());
                        if !t.description.is_empty() {
                            let first = t.description.lines().next().unwrap_or("");
                            println!("       {}", first.dimmed());
                        }
                    }
                }

                if !sf.completed.is_empty() {
                    println!();
                    println!("Completed ({}):", sf.completed.len());
                    for c in sf.completed.iter().take(5) {
                        println!("  {} {}", "✓".green(), c.dimmed());
                    }
                    if sf.completed.len() > 5 {
                        println!("  ... and {} more", sf.completed.len() - 5);
                    }
                }

                if !sf.roadmap.is_empty() {
                    println!();
                    println!("Roadmap:");
                    for r in &sf.roadmap {
                        let check = if r.done { "✓".green() } else { "○".dimmed() };
                        println!("  {} {}", check, r.description);
                    }
                }
            }
        }
        Commands::Status { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load_or_init(&dir)?;
            project::show_status(&proj)?;
        }
    }

    Ok(())
}
