use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "subspace", version, about = "AI coding agent swarm orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a project for Subspace (creates .subspace/)
    Init {
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Run planner only — output tasks without executing
    Plan {
        /// Goal to plan for
        goal: String,
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Full plan/execute/merge loop
    Run {
        /// What to build (the goal)
        goal: String,
        #[arg(long)]
        project_dir: Option<String>,
        /// Max iterations (default: 10)
        #[arg(long, default_value = "10")]
        max_iter: u32,
        /// Max parallel agents (default: 3)
        #[arg(long, default_value = "3")]
        parallel: u32,
    },

    /// Manage the agent swarm
    Swarm {
        #[command(subcommand)]
        command: SwarmCommands,
    },

    /// Show project status
    Status {
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Show file conflicts between agents
    Conflicts {
        #[arg(long)]
        project_dir: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SwarmCommands {
    /// Launch parallel agents for a goal
    Launch {
        /// What to build
        goal: String,
        #[arg(long)]
        project_dir: Option<String>,
        /// Number of executor agents (default: 3)
        #[arg(long, default_value = "3")]
        agents: u32,
    },

    /// Show all agents and their status
    Status,

    /// View agent output
    Logs {
        /// Agent ID (e.g. executor-1)
        agent_id: String,
    },

    /// Merge completed agent branches (Haiku)
    Merge {
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Clean up worktrees and state
    Clean {
        #[arg(long)]
        project_dir: Option<String>,
    },
}
