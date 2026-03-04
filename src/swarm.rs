use anyhow::Result;
use chrono::Utc;
use colored::*;
use std::path::Path;

use crate::agent;
use crate::config::{self, AgentRole, AgentState, AgentStatus, SwarmState};
use crate::conflict;
use crate::executor;
use crate::merger;
use crate::planner;
use crate::project::{self, Project};
use crate::tmux;
use crate::worktree;

/// Run the full plan/execute/merge loop
pub async fn run_loop(proj: &Project, max_iter: u32, parallel: u32, tool: &str) -> Result<()> {
    tmux::check_tmux()?;

    let session = project::session_name(&proj.dir);
    let base_branch = worktree::current_branch(&proj.dir)?;

    println!("{}", "═══════════════════════════════════════".bold());
    println!("  {} {}", "Subspace".bold().cyan(), "— Agent Swarm");
    println!("{}", "═══════════════════════════════════════".bold());
    println!("Tool:       {}", tool.yellow());
    println!("Project:    {}", proj.dir.display().to_string().cyan());
    println!("Stack:      {}", proj.stack.yellow());
    println!("Base:       {}", base_branch.green());
    println!("Parallel:   {}", parallel);
    println!("Iterations: {}", max_iter);
    println!("Isolation:  {}", "git worktrees".green());
    println!("Merger:     {}", "Sonnet (auto)".blue());
    println!();

    let mut state = SwarmState::new(
        &session,
        &proj.dir.to_string_lossy(),
        &proj.stack,
    );
    config::set_active_project(&proj.dir)?;

    for i in 1..=max_iter {
        println!();
        println!("{}", format!("═══ Iteration {} of {} ═══", i, max_iter).bold());
        state.iteration = i;

        // Phase A: Planning
        println!();
        println!("{}", "── Phase A: Planning ──".dimmed());
        let prompt_file = planner::write_prompt(proj, i)?;

        let planner_state = agent::spawn(
            &session,
            "planner",
            AgentRole::Planner,
            tool,
            &prompt_file,
            &proj.dir.to_string_lossy(),
            None,
        )?;
        state.agents.retain(|a| a.role != AgentRole::Planner);
        state.agents.push(planner_state);
        config::save_state(&proj.dir, &state)?;

        // Wait for planner
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            if agent::is_done(&session, "planner")? || !agent::is_running(&session, "planner") {
                break;
            }
            print!(".");
        }
        println!();

        let plan_output = agent::get_output(&session, "planner", 500)?;

        if let Some(p) = state.agents.iter_mut().find(|a| a.id == "planner") {
            p.status = AgentStatus::Completed;
            p.finished_at = Some(Utc::now());
        }

        // Check completion
        if planner::is_complete(&plan_output) {
            println!();
            println!("{}", "═══════════════════════════════════════".green().bold());
            println!("  {} All goals achieved at iteration {}!", "✓".green(), i);
            println!("{}", "═══════════════════════════════════════".green().bold());
            config::save_state(&proj.dir, &state)?;
            return Ok(());
        }

        // Parse tasks
        let tasks = planner::parse_tasks(&plan_output);
        let task_count = tasks.len().min(parallel as usize);

        println!(
            "Planner produced {} task(s), spawning {} in worktrees",
            tasks.len(),
            task_count
        );

        // Phase B: Execution (each agent in its own worktree)
        println!();
        println!("{}", "── Phase B: Execution (worktree-isolated) ──".dimmed());

        let proj = project::load(&proj.dir)?;
        let mut executor_ids = Vec::new();

        for task in tasks.iter().take(task_count) {
            let (eid, wt_path) = executor::spawn_executor(&session, &proj, task, tool, &base_branch)?;
            println!(
                "  {} {} → {} (worktree: {})",
                "→".green(),
                eid.bold(),
                task.title.cyan(),
                wt_path.dimmed()
            );

            state.agents.push(AgentState {
                id: eid.clone(),
                role: AgentRole::Executor,
                tmux_window: eid.clone(),
                status: AgentStatus::Running,
                started_at: Utc::now(),
                finished_at: None,
                task: Some(config::TaskInfo {
                    title: task.title.clone(),
                    files: task.files.clone(),
                }),
                files_modified: Vec::new(),
            });

            executor_ids.push(eid);
        }
        config::save_state(&proj.dir, &state)?;

        // Wait for all executors
        println!();
        println!("Waiting for executors...");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let all_done = executor_ids.iter().all(|eid| {
                agent::is_done(&session, eid).unwrap_or(false)
                    || !agent::is_running(&session, eid)
            });
            if all_done {
                break;
            }
            let running: Vec<_> = executor_ids
                .iter()
                .filter(|eid| agent::is_running(&session, eid))
                .collect();
            print!("\r  {} agent(s) still running...", running.len());
        }
        println!();

        // Update executor statuses
        for eid in &executor_ids {
            if let Some(a) = state.agents.iter_mut().find(|a| a.id == *eid) {
                a.status = AgentStatus::Completed;
                a.finished_at = Some(Utc::now());
            }
        }
        config::save_state(&proj.dir, &state)?;

        // Phase C: Merge with Sonnet
        println!();
        println!("{}", "── Phase C: Merge (Sonnet) ──".dimmed());

        let completed: Vec<String> = executor_ids.clone();
        if !completed.is_empty() {
            for id in &completed {
                let diff = worktree::diff_from_base(&proj.dir, id, &base_branch)?;
                println!("  {} subspace/{}", "📝".to_string(), id);
                for line in diff.lines().take(5) {
                    println!("     {}", line.dimmed());
                }
            }

            let merger_id = merger::spawn_merger(&session, &proj, &base_branch, &completed)?;

            // Wait for merger
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                if agent::is_done(&session, &merger_id)? || !agent::is_running(&session, &merger_id) {
                    break;
                }
                print!(".");
            }
            println!();
            println!("  {} Merge complete", "✓".green());
        }

        println!("{}", format!("── Iteration {} complete ──", i).dimmed());
    }

    println!();
    println!(
        "{} Reached max iterations ({}). Check progress.md",
        "⚠".yellow(),
        max_iter
    );
    Ok(())
}

/// Launch a swarm (worktree-isolated agents)
pub async fn launch(proj: &Project, agent_count: u32, tool: &str) -> Result<()> {
    tmux::check_tmux()?;

    let session = project::session_name(&proj.dir);
    let base_branch = worktree::current_branch(&proj.dir)?;

    println!("{}", "═══ Subspace Swarm Launch ═══".bold());
    println!("Agents:     {}", agent_count);
    println!("Base:       {}", base_branch.green());
    println!("Isolation:  {}", "git worktrees".green());
    println!();

    // Run planner first
    let prompt_file = planner::write_prompt(proj, 1)?;
    let _planner = agent::spawn(
        &session,
        "planner",
        AgentRole::Planner,
        tool,
        &prompt_file,
        &proj.dir.to_string_lossy(),
        None,
    )?;

    println!("Planner spawned, waiting for plan...");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        if agent::is_done(&session, "planner")? || !agent::is_running(&session, "planner") {
            break;
        }
    }

    let plan_output = agent::get_output(&session, "planner", 500)?;
    let tasks = planner::parse_tasks(&plan_output);

    let mut state = SwarmState::new(&session, &proj.dir.to_string_lossy(), &proj.stack);
    state.iteration = 1;
    config::set_active_project(&proj.dir)?;

    let spawn_count = tasks.len().min(agent_count as usize);
    for task in tasks.iter().take(spawn_count) {
        let (eid, wt_path) = executor::spawn_executor(&session, proj, task, tool, &base_branch)?;
        println!("  {} {} — {} ({})", "✓".green(), eid, task.title.cyan(), wt_path.dimmed());

        state.agents.push(AgentState {
            id: eid.clone(),
            role: AgentRole::Executor,
            tmux_window: eid.clone(),
            status: AgentStatus::Running,
            started_at: Utc::now(),
            finished_at: None,
            task: Some(config::TaskInfo {
                title: task.title.clone(),
                files: task.files.clone(),
            }),
            files_modified: Vec::new(),
        });
    }

    config::save_state(&proj.dir, &state)?;
    println!();
    println!("Swarm running in worktrees. Commands:");
    println!("  {} swarm status           — see agent status", "subspace".bold());
    println!("  {} swarm logs executor-1   — tail output", "subspace".bold());
    println!("  {} swarm merge             — merge with Sonnet", "subspace".bold());
    println!("  {} swarm clean             — remove worktrees", "subspace".bold());
    Ok(())
}

/// Show status of all running agents
pub async fn status() -> Result<()> {
    let found = config::find_active_state()?;
    match found {
        None => {
            println!("{}", "No active swarm found.".dimmed());
        }
        Some((_dir, state)) => {
            println!("{}", "═══ Subspace Swarm Status ═══".bold());
            println!("Session:   {}", state.session_name.green());
            println!("Project:   {}", state.project_dir.cyan());
            println!("Stack:     {}", state.stack.yellow());
            println!("Iteration: {}", state.iteration);
            println!();

            if state.agents.is_empty() {
                println!("{}", "No agents registered.".dimmed());
            } else {
                for a in &state.agents {
                    let status_str = match a.status {
                        AgentStatus::Running => {
                            let live = agent::is_running(&state.session_name, &a.id);
                            if live { "● running".green() } else { "○ stopped".yellow() }
                        }
                        AgentStatus::Completed => "✓ completed".blue(),
                        AgentStatus::Failed => "✗ failed".red(),
                        AgentStatus::Pending => "◌ pending".dimmed(),
                    };

                    let role = match a.role {
                        AgentRole::Planner => "🧠",
                        AgentRole::Executor => "🔨",
                    };

                    println!("  {} {} [{}]", role, a.id.bold(), status_str);
                    if let Some(task) = &a.task {
                        println!("    Task: {}", task.title.cyan());
                        if !task.files.is_empty() {
                            println!("    Files: {}", task.files.join(", ").dimmed());
                        }
                    }

                    // Show worktree branch info for executors
                    if a.role == AgentRole::Executor {
                        println!("    Branch: {}", format!("subspace/{}", a.id).dimmed());
                    }
                }
            }

            if !state.conflicts.is_empty() {
                println!();
                println!("{}", "⚠ Conflicts:".red());
                for c in &state.conflicts {
                    println!("  {} — {}", c.file.yellow(), c.agents.join(", "));
                }
            }
        }
    }
    Ok(())
}

/// Stream logs from an agent
pub async fn logs(agent_id: &str, follow: bool) -> Result<()> {
    let found = config::find_active_state()?;
    let (_, state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;

    let output = agent::get_output(&state.session_name, agent_id, if follow { 200 } else { 100 })?;
    println!("{}", output);

    if follow {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let new_output = agent::get_output(&state.session_name, agent_id, 20)?;
            print!("{}", new_output);
            if !agent::is_running(&state.session_name, agent_id) {
                println!("\n{}", "[Agent finished]".dimmed());
                break;
            }
        }
    }
    Ok(())
}

/// Send instruction to a running agent
pub async fn steer(agent_id: &str, message: &str) -> Result<()> {
    let found = config::find_active_state()?;
    let (_, state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;

    agent::steer(&state.session_name, agent_id, message)?;
    println!("{} Sent to {}: {}", "→".green(), agent_id.cyan(), message);
    Ok(())
}

/// Kill a specific agent
pub async fn kill(agent_id: &str) -> Result<()> {
    let found = config::find_active_state()?;
    let (dir, mut state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;

    agent::kill(&state.session_name, agent_id)?;

    if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent_id) {
        a.status = AgentStatus::Failed;
        a.finished_at = Some(Utc::now());
    }
    config::save_state(&dir, &state)?;

    println!("{} Killed agent: {}", "✗".red(), agent_id);
    Ok(())
}

/// Kill all agents
pub async fn kill_all() -> Result<()> {
    let found = config::find_active_state()?;
    let (dir, mut state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;

    tmux::kill_session(&state.session_name)?;

    for a in &mut state.agents {
        if a.status == AgentStatus::Running {
            a.status = AgentStatus::Failed;
            a.finished_at = Some(Utc::now());
        }
    }
    config::save_state(&dir, &state)?;

    println!("{} Killed all agents in session {}", "✗".red(), state.session_name);
    Ok(())
}

/// Clean up worktrees and branches
pub fn clean(project_dir: &Path) -> Result<()> {
    println!("Cleaning up worktrees and subspace branches...");
    worktree::remove_all(project_dir)?;

    // Also remove state
    let state_path = project_dir.join(".subspace").join("state.json");
    if state_path.exists() {
        std::fs::remove_file(&state_path)?;
    }

    println!("{} Cleaned up all worktrees and branches", "✓".green());
    Ok(())
}
