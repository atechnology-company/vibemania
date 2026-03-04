use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "subspace", version, about = "Autonomous AI project completion engine — takes projects from 0% to 95%")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize .subspace/ in a project
    Init {
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Audit & plan only — show what the swarm would do
    Plan {
        /// Optional focus goal (omit for autonomous discovery)
        goal: Option<String>,
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Full autonomous loop: discover → audit → plan → execute → merge → repeat
    Run {
        /// Optional focus goal (omit to let the planner decide)
        goal: Option<String>,
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

    /// Show and manage tasks in subspace.md
    Tasks {
        #[arg(long)]
        project_dir: Option<String>,
        /// Add a new task
        #[arg(long)]
        add: Option<String>,
        /// Priority: critical, high, medium, low, nice
        #[arg(long, default_value = "medium")]
        priority: String,
    },

    /// Show project status
    Status {
        #[arg(long)]
        project_dir: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SwarmCommands {
    /// Launch agents (plan once, execute, wait for merge)
    Launch {
        /// Optional focus goal
        goal: Option<String>,
        #[arg(long)]
        project_dir: Option<String>,
        #[arg(long, default_value = "3")]
        agents: u32,
    },

    /// Show agent status
    Status,

    /// View agent output
    Logs {
        agent_id: String,
    },

    /// Merge completed branches (Haiku)
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
