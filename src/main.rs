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
use cli::{Cli, Commands, SwarmCommands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            project::init(&dir)?;
        }
        Commands::Plan { project_dir, tool } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load(&dir)?;
            planner::run_planner(&proj, &tool.unwrap_or_default()).await?;
        }
        Commands::Run { project_dir, max_iter, parallel, tool } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load(&dir)?;
            let tool = tool.unwrap_or_default();
            swarm::run_loop(&proj, max_iter, parallel, &tool).await?;
        }
        Commands::Swarm { command } => match command {
            SwarmCommands::Launch { project_dir, agents, tool } => {
                let dir = project::resolve_dir(project_dir)?;
                let proj = project::load(&dir)?;
                let tool = tool.unwrap_or_default();
                swarm::launch(&proj, agents, &tool).await?;
            }
            SwarmCommands::Status => {
                swarm::status().await?;
            }
            SwarmCommands::Logs { agent_id, follow } => {
                swarm::logs(&agent_id, follow).await?;
            }
            SwarmCommands::Steer { agent_id, message } => {
                swarm::steer(&agent_id, &message).await?;
            }
            SwarmCommands::Kill { agent_id, all } => {
                if all {
                    swarm::kill_all().await?;
                } else if let Some(id) = agent_id {
                    swarm::kill(&id).await?;
                } else {
                    eprintln!("Specify an agent ID or --all");
                }
            }
            SwarmCommands::Merge { project_dir } => {
                let dir = project::resolve_dir(project_dir)?;
                let proj = project::load(&dir)?;
                merger::run_merge(&proj).await?;
            }
            SwarmCommands::Clean { project_dir } => {
                let dir = project::resolve_dir(project_dir)?;
                swarm::clean(&dir)?;
            }
        },
        Commands::Status { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load(&dir)?;
            project::show_status(&proj)?;
        }
        Commands::Conflicts { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load(&dir)?;
            conflict::show(&proj)?;
        }
    }

    Ok(())
}
