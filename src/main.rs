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
                let _proj = project::load_or_init(&dir)?;
                swarm::merge(&dir).await?;
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
    }

    Ok(())
}
