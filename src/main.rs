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
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { project_dir } => {
            let dir = project::resolve_dir(project_dir)?;
            project::init(&dir)?;
        }
        Commands::Plan { project_dir, tool: _ } => {
            let dir = project::resolve_dir(project_dir)?;
            let proj = project::load(&dir)?;
            let prompt = planner::build_prompt(&proj, 1);
            let result = acp::run_prompt(&dir, &prompt).await?;
            println!("{}", result.text);
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
                // Run merge via ACP
                let state = config::load_state(&dir)?;
                if let Some(state) = state {
                    let base_branch = worktree::current_branch(&dir)?;
                    let completed: Vec<String> = state
                        .agents
                        .iter()
                        .filter(|a| a.role == config::AgentRole::Executor && a.status == config::AgentStatus::Completed)
                        .map(|a| a.id.clone())
                        .collect();

                    if completed.is_empty() {
                        println!("No completed agents to merge.");
                        return Ok(());
                    }

                    // Collect branch details
                    let mut branch_details = String::new();
                    for agent_id in &completed {
                        let diff = worktree::diff_from_base(&dir, agent_id, &base_branch).unwrap_or_default();
                        let full_diff = worktree::full_diff(&dir, agent_id, &base_branch).unwrap_or_default();
                        let commits = worktree::branch_log(&dir, agent_id, &base_branch).unwrap_or_default();
                        branch_details.push_str(&format!(
                            "### Branch: subspace/{}\n#### Commits:\n```\n{}\n```\n#### Changed:\n```\n{}\n```\n#### Diff:\n```diff\n{}\n```\n---\n\n",
                            agent_id, commits.trim(), diff.trim(), full_diff.trim()
                        ));
                    }

                    println!("Merging {} branch(es) with Sonnet...", completed.len());
                    // Build merge prompt inline
                    let merge_prompt = format!(
                        "# Merge Agent\n\nMerge these branches into `{}`:\n\n{}\n\n## Instructions\n\nFor each branch: `git merge subspace/<id> --no-ff`\nResolve conflicts intelligently. Run quality checks for {} stack. Commit result.\n\n{}",
                        base_branch,
                        completed.iter().map(|id| format!("- subspace/{}", id)).collect::<Vec<_>>().join("\n"),
                        proj.stack,
                        branch_details,
                    );
                    let result = acp::run_prompt(&dir, &merge_prompt).await?;
                    println!("{}", result.text);
                } else {
                    println!("No active swarm state.");
                }
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
