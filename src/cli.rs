use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "subspace", version, about = "AI coding agent swarm orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a project for Subspace (creates goals.md, .subspace/)
    Init {
        /// Project directory (default: current dir)
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Run planner phase only — analyze goals and output tasks
    Plan {
        #[arg(long)]
        project_dir: Option<String>,
        /// AI tool: claude, amp (default: claude)
        #[arg(long)]
        tool: Option<String>,
    },

    /// Full plan/execute loop (worktree-isolated agents + Sonnet merger)
    Run {
        #[arg(long)]
        project_dir: Option<String>,
        /// Max iterations (default: 10)
        #[arg(long, default_value = "10")]
        max_iter: u32,
        /// Max parallel agents (default: 3)
        #[arg(long, default_value = "3")]
        parallel: u32,
        /// AI tool: claude, amp (default: claude)
        #[arg(long)]
        tool: Option<String>,
    },

    /// Manage the agent swarm
    Swarm {
        #[command(subcommand)]
        command: SwarmCommands,
    },

    /// Show project status (progress, agents, iteration)
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
    /// Launch parallel agent swarm (each in its own worktree)
    Launch {
        #[arg(long)]
        project_dir: Option<String>,
        /// Number of executor agents (default: 3)
        #[arg(long, default_value = "3")]
        agents: u32,
        /// AI tool: claude, amp (default: claude)
        #[arg(long)]
        tool: Option<String>,
    },

    /// Show all running agents and their status
    Status,

    /// Stream/tail logs from a specific agent
    Logs {
        /// Agent ID (e.g. executor-1)
        agent_id: String,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// Send instruction to a specific agent
    Steer {
        /// Agent ID
        agent_id: String,
        /// Message to send
        message: String,
    },

    /// Kill agent(s)
    Kill {
        /// Agent ID to kill
        agent_id: Option<String>,
        /// Kill all agents
        #[arg(long)]
        all: bool,
    },

    /// Merge all completed agent branches using Sonnet
    Merge {
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Clean up worktrees and agent branches
    Clean {
        #[arg(long)]
        project_dir: Option<String>,
    },
}
