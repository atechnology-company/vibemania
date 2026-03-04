use anyhow::Result;
use chrono::Utc;
use colored::*;

use crate::agent;
use crate::config::{self, AgentRole, AgentState, AgentStatus, SwarmState};
use crate::conflict;
use crate::executor;
use crate::planner;
use crate::project::{self, Project};
use crate::tmux;

/// Run the full plan/execute loop
pub async fn run_loop(proj: &Project, max_iter: u32, parallel: u32, tool: &str) -> Result<()> {
    tmux::check_tmux()?;

    let session = project::session_name(&proj.dir);

    println!("{}", "═══════════════════════════════════════".bold());
    println!("  {} {}", "Subspace".bold().cyan(), "— Agent Swarm");
    println!("{}", "═══════════════════════════════════════".bold());
    println!("Tool:       {}", tool.yellow());
    println!("Project:    {}", proj.dir.display().to_string().cyan());
    println!("Stack:      {}", proj.stack.yellow());
    println!("Parallel:   {}", parallel);
    println!("Iterations: {}", max_iter);
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

        // Update planner status
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
            "Planner produced {} task(s), running {} in parallel",
            tasks.len(),
            task_count
        );

        // Phase B: Execution
        println!();
        println!("{}", "── Phase B: Execution ──".dimmed());

        // Reload project to get fresh progress
        let proj = project::load(&proj.dir)?;

        let mut executor_ids = Vec::new();
        for task in tasks.iter().take(task_count) {
            println!(
                "  {} Spawning executor for: {}",
                "→".green(),
                task.title.cyan()
            );
            let eid = executor::spawn_executor(&session, &proj, task, tool)?;

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

        // Phase C: Conflict detection
        let conflicts = conflict::detect(&state);
        if !conflicts.is_empty() {
            println!();
            println!(
                "{} {} file conflict(s) detected!",
                "⚠".red(),
                conflicts.len()
            );
            for c in &conflicts {
                println!("  {} — {}", c.file.yellow(), c.agents.join(", "));
            }
        }
        state.conflicts = conflicts;

        config::save_state(&proj.dir, &state)?;
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

/// Launch a swarm of agents for the current plan
pub async fn launch(proj: &Project, agent_count: u32, tool: &str) -> Result<()> {
    tmux::check_tmux()?;

    let session = project::session_name(&proj.dir);
    println!("Launching {} executor agents in tmux session: {}", agent_count, session.green());

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
        let eid = executor::spawn_executor(&session, proj, task, tool)?;
        println!("  {} {} — {}", "✓".green(), eid, task.title.cyan());

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
    println!("Swarm running. Use {} to monitor.", "subspace swarm status".bold());
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
                        AgentRole::Planner => "🧠 planner".to_string(),
                        AgentRole::Executor => "🔨 executor".to_string(),
                    };

                    println!("  {} {} [{}]", a.id.bold(), role, status_str);
                    if let Some(task) = &a.task {
                        println!("    Task: {}", task.title.cyan());
                        if !task.files.is_empty() {
                            println!("    Files: {}", task.files.join(", ").dimmed());
                        }
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
