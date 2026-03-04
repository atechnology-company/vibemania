//! Swarm orchestrator — coordinates multiple ACP agents with worktree isolation.
//!
//! Flow:
//! 1. Planner agent analyzes goals → produces independent tasks
//! 2. Executor agents run in parallel, each in its own git worktree
//! 3. Merger agent (Sonnet) reviews all diffs and merges branches
//!
//! All agents communicate via ACP (structured JSON-RPC, auto-approve permissions).

use anyhow::Result;
use chrono::Utc;
use colored::*;
use std::path::{Path, PathBuf};

use crate::acp;
use crate::config::{self, AgentRole, AgentState, AgentStatus, SwarmState, TaskInfo};
use crate::planner;
use crate::project::{self, Project};
use crate::worktree;

/// Run the full plan/execute/merge loop using ACP agents
pub async fn run_loop(proj: &Project, max_iter: u32, parallel: u32, _tool: &str) -> Result<()> {
    // Verify ACP binary exists
    acp::find_acp_binary()?;

    let session_name = project::session_name(&proj.dir);
    let base_branch = worktree::current_branch(&proj.dir)?;

    println!("{}", "═══════════════════════════════════════════".bold());
    println!("  {} {}", "Subspace".bold().cyan(), "— Agent Swarm Orchestrator");
    println!("{}", "═══════════════════════════════════════════".bold());
    println!("Backend:    {}", "ACP (claude-agent-acp)".green());
    println!("Project:    {}", proj.dir.display().to_string().cyan());
    println!("Stack:      {}", proj.stack.yellow());
    println!("Base:       {}", base_branch.green());
    println!("Parallel:   {}", parallel);
    println!("Iterations: {}", max_iter);
    println!("Isolation:  {}", "git worktrees".green());
    println!("Merger:     {}", "Sonnet (auto)".blue());
    println!("Permissions: {}", "bypassed (auto-approve)".red());
    println!();

    let mut state = SwarmState::new(&session_name, &proj.dir.to_string_lossy(), &proj.stack);
    config::set_active_project(&proj.dir)?;

    for i in 1..=max_iter {
        println!();
        println!("{}", format!("═══ Iteration {} of {} ═══", i, max_iter).bold());
        state.iteration = i;

        // ──────────────────────────────────────────
        // Phase A: Planning
        // ──────────────────────────────────────────
        println!();
        println!("{}", "── Phase A: Planning ──".dimmed());
        println!("  Analyzing goals and progress...");

        // Reload project for fresh progress
        let proj = project::load(&proj.dir)?;
        let planner_prompt = planner::build_prompt(&proj, i);

        let plan_result = acp::run_prompt(&proj.dir, &planner_prompt).await?;

        state.agents.retain(|a| a.role != AgentRole::Planner);
        state.agents.push(AgentState {
            id: "planner".to_string(),
            role: AgentRole::Planner,
            tmux_window: String::new(),
            status: AgentStatus::Completed,
            started_at: Utc::now(),
            finished_at: Some(Utc::now()),
            task: None,
            files_modified: plan_result.files_modified.clone(),
        });
        config::save_state(&proj.dir, &state)?;

        // Save plan output
        let subspace_dir = proj.dir.join(".subspace");
        std::fs::create_dir_all(&subspace_dir)?;
        std::fs::write(subspace_dir.join("plan.md"), &plan_result.text)?;

        // Check completion
        if planner::is_complete(&plan_result.text) {
            println!();
            println!("{}", "═══════════════════════════════════════════".green().bold());
            println!("  {} All goals achieved at iteration {}!", "✓".green(), i);
            println!("{}", "═══════════════════════════════════════════".green().bold());
            config::save_state(&proj.dir, &state)?;
            return Ok(());
        }

        // Parse tasks
        let tasks = planner::parse_tasks(&plan_result.text);
        let task_count = tasks.len().min(parallel as usize);

        println!("  Planner produced {} task(s), running {} in parallel", tasks.len(), task_count);
        for task in tasks.iter().take(task_count) {
            println!("    {} {}", "→".cyan(), task.title);
            if !task.files.is_empty() {
                println!("      files: {}", task.files.join(", ").dimmed());
            }
        }

        // ──────────────────────────────────────────
        // Phase B: Parallel Execution (worktree-isolated ACP agents)
        // ──────────────────────────────────────────
        println!();
        println!("{}", "── Phase B: Parallel Execution ──".dimmed());

        // Create worktrees and spawn agents concurrently
        let mut handles = Vec::new();
        let mut agent_infos = Vec::new();

        for task in tasks.iter().take(task_count) {
            let agent_id = format!("executor-{}", task.id);

            // Create worktree
            let wt_dir = worktree::create(&proj.dir, &agent_id, &base_branch)?;
            println!("  {} {} → worktree created", "✓".green(), agent_id.bold());

            // Build executor prompt with task context
            let executor_prompt = build_executor_prompt(&proj, task);

            // Save prompt for debugging
            std::fs::write(
                subspace_dir.join(format!("executor-prompt-{}.md", task.id)),
                &executor_prompt,
            )?;

            agent_infos.push((agent_id.clone(), task.title.clone(), task.files.clone()));

            // Spawn ACP agent in its own tokio task
            let wt_path = wt_dir.clone();
            let handle = tokio::spawn(async move {
                let result = acp::run_prompt(&wt_path, &executor_prompt).await;
                (agent_id, result)
            });
            handles.push(handle);
        }

        // Register agents in state
        for (agent_id, title, files) in &agent_infos {
            state.agents.push(AgentState {
                id: agent_id.clone(),
                role: AgentRole::Executor,
                tmux_window: String::new(),
                status: AgentStatus::Running,
                started_at: Utc::now(),
                finished_at: None,
                task: Some(TaskInfo {
                    title: title.clone(),
                    files: files.clone(),
                }),
                files_modified: Vec::new(),
            });
        }
        config::save_state(&proj.dir, &state)?;

        // Wait for all agents to complete
        println!();
        println!("  Waiting for {} agent(s)...", handles.len());

        let mut completed_agents = Vec::new();
        let mut all_files_modified = Vec::new();

        for handle in handles {
            match handle.await {
                Ok((agent_id, Ok(output))) => {
                    println!("  {} {} completed ({} tool calls, {} files modified)",
                        "✓".green(),
                        agent_id.bold(),
                        output.tool_calls.len(),
                        output.files_modified.len(),
                    );

                    // Save agent output
                    std::fs::write(
                        subspace_dir.join(format!("{}-output.md", agent_id)),
                        &output.text,
                    )?;

                    if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent_id) {
                        a.status = AgentStatus::Completed;
                        a.finished_at = Some(Utc::now());
                        a.files_modified = output.files_modified.clone();
                    }

                    all_files_modified.extend(output.files_modified);
                    completed_agents.push(agent_id);
                }
                Ok((agent_id, Err(e))) => {
                    println!("  {} {} failed: {}", "✗".red(), agent_id.bold(), e);
                    if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent_id) {
                        a.status = AgentStatus::Failed;
                        a.finished_at = Some(Utc::now());
                    }
                }
                Err(e) => {
                    println!("  {} agent panicked: {}", "✗".red(), e);
                }
            }
        }
        config::save_state(&proj.dir, &state)?;

        if completed_agents.is_empty() {
            println!("  {} No agents completed successfully, skipping merge", "⚠".yellow());
            continue;
        }

        // ──────────────────────────────────────────
        // Phase C: Merge (Sonnet agent reviews all diffs)
        // ──────────────────────────────────────────
        println!();
        println!("{}", "── Phase C: Merge (Sonnet) ──".dimmed());

        // Collect diffs from all completed agent branches
        let mut branch_summaries = String::new();
        for agent_id in &completed_agents {
            let diff_stat = worktree::diff_from_base(&proj.dir, agent_id, &base_branch)
                .unwrap_or_default();
            let full_diff = worktree::full_diff(&proj.dir, agent_id, &base_branch)
                .unwrap_or_default();
            let commits = worktree::branch_log(&proj.dir, agent_id, &base_branch)
                .unwrap_or_default();

            // Read the agent's output summary
            let agent_output = std::fs::read_to_string(
                subspace_dir.join(format!("{}-output.md", agent_id))
            ).unwrap_or_default();

            branch_summaries.push_str(&format!(
                "### Branch: subspace/{}\n\n\
                 #### What this agent did:\n{}\n\n\
                 #### Commits:\n```\n{}\n```\n\n\
                 #### Files Changed:\n```\n{}\n```\n\n\
                 #### Full Diff:\n```diff\n{}\n```\n\n---\n\n",
                agent_id,
                truncate(&agent_output, 2000),
                commits.trim(),
                diff_stat.trim(),
                truncate(&full_diff, 8000),
            ));

            println!("  {} subspace/{} — {} files changed",
                "📝",
                agent_id,
                diff_stat.lines().count().saturating_sub(1),
            );
        }

        let merge_prompt = build_merge_prompt(&proj, &base_branch, &completed_agents, &branch_summaries);

        // Save merge prompt for debugging
        std::fs::write(subspace_dir.join("merger-prompt.md"), &merge_prompt)?;

        println!("  Merger agent analyzing diffs and merging...");

        // Merger runs in the main repo (not a worktree)
        let merge_result = acp::run_prompt(&proj.dir, &merge_prompt).await?;

        // Save merge output
        std::fs::write(subspace_dir.join("merger-output.md"), &merge_result.text)?;

        println!("  {} Merge complete ({} tool calls)",
            "✓".green(),
            merge_result.tool_calls.len(),
        );

        // Detect conflicts (files modified by multiple agents)
        let conflicts = detect_conflicts(&state);
        if !conflicts.is_empty() {
            println!();
            println!("  {} {} potential conflict(s) detected:", "⚠".yellow(), conflicts.len());
            for (file, agents) in &conflicts {
                println!("    {} — modified by: {}", file.yellow(), agents.join(", "));
            }
        }
        state.conflicts = conflicts
            .into_iter()
            .map(|(file, agents)| config::FileConflict { file, agents })
            .collect();

        config::save_state(&proj.dir, &state)?;
        println!();
        println!("{}", format!("── Iteration {} complete ──", i).dimmed());
    }

    println!();
    println!("{} Reached max iterations ({}). Check progress.md", "⚠".yellow(), max_iter);
    Ok(())
}

/// Launch a swarm without the loop (plan once, execute, then user decides)
pub async fn launch(proj: &Project, agent_count: u32, _tool: &str) -> Result<()> {
    acp::find_acp_binary()?;

    let base_branch = worktree::current_branch(&proj.dir)?;
    let session_name = project::session_name(&proj.dir);

    println!("{}", "═══ Subspace Swarm Launch ═══".bold());
    println!("Backend:    {}", "ACP".green());
    println!("Agents:     {}", agent_count);
    println!("Base:       {}", base_branch.green());
    println!();

    // Plan
    println!("Planning...");
    let planner_prompt = planner::build_prompt(proj, 1);
    let plan_result = acp::run_prompt(&proj.dir, &planner_prompt).await?;

    let tasks = planner::parse_tasks(&plan_result.text);
    let task_count = tasks.len().min(agent_count as usize);

    println!("Got {} task(s), spawning {} agents:", tasks.len(), task_count);

    let mut state = SwarmState::new(&session_name, &proj.dir.to_string_lossy(), &proj.stack);
    state.iteration = 1;
    config::set_active_project(&proj.dir)?;

    // Spawn agents
    let mut handles = Vec::new();
    let subspace_dir = proj.dir.join(".subspace");
    std::fs::create_dir_all(&subspace_dir)?;

    for task in tasks.iter().take(task_count) {
        let agent_id = format!("executor-{}", task.id);
        let wt_dir = worktree::create(&proj.dir, &agent_id, &base_branch)?;
        let prompt = build_executor_prompt(proj, task);

        println!("  {} {} — {}", "→".green(), agent_id.bold(), task.title.cyan());

        state.agents.push(AgentState {
            id: agent_id.clone(),
            role: AgentRole::Executor,
            tmux_window: String::new(),
            status: AgentStatus::Running,
            started_at: Utc::now(),
            finished_at: None,
            task: Some(TaskInfo {
                title: task.title.clone(),
                files: task.files.clone(),
            }),
            files_modified: Vec::new(),
        });

        let wt_path = wt_dir.clone();
        let handle = tokio::spawn(async move {
            let result = acp::run_prompt(&wt_path, &prompt).await;
            (agent_id, result)
        });
        handles.push(handle);
    }
    config::save_state(&proj.dir, &state)?;

    println!();
    println!("Agents running. Waiting for completion...");

    for handle in handles {
        match handle.await {
            Ok((agent_id, Ok(output))) => {
                println!("  {} {} done", "✓".green(), agent_id);
                if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent_id) {
                    a.status = AgentStatus::Completed;
                    a.finished_at = Some(Utc::now());
                    a.files_modified = output.files_modified;
                }
            }
            Ok((agent_id, Err(e))) => {
                println!("  {} {} failed: {}", "✗".red(), agent_id, e);
                if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent_id) {
                    a.status = AgentStatus::Failed;
                    a.finished_at = Some(Utc::now());
                }
            }
            Err(e) => println!("  {} panic: {}", "✗".red(), e),
        }
    }
    config::save_state(&proj.dir, &state)?;

    println!();
    println!("Run {} to merge branches with Sonnet", "subspace swarm merge".bold());
    Ok(())
}

/// Show status of the current swarm
pub async fn status() -> Result<()> {
    let found = config::find_active_state()?;
    match found {
        None => println!("{}", "No active swarm found.".dimmed()),
        Some((_dir, state)) => {
            println!("{}", "═══ Subspace Swarm Status ═══".bold());
            println!("Session:   {}", state.session_name.green());
            println!("Project:   {}", state.project_dir.cyan());
            println!("Stack:     {}", state.stack.yellow());
            println!("Iteration: {}", state.iteration);
            println!();

            for a in &state.agents {
                let status_str = match a.status {
                    AgentStatus::Running => "● running".green(),
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
                }
                if !a.files_modified.is_empty() {
                    println!("    Modified: {}", a.files_modified.join(", ").dimmed());
                }
                if a.role == AgentRole::Executor {
                    println!("    Branch: {}", format!("subspace/{}", a.id).dimmed());
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

/// View agent output logs
pub async fn logs(agent_id: &str, _follow: bool) -> Result<()> {
    let found = config::find_active_state()?;
    let (dir, _state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;
    let output_path = PathBuf::from(&dir).join(".subspace").join(format!("{}-output.md", agent_id));
    if output_path.exists() {
        let content = std::fs::read_to_string(&output_path)?;
        println!("{}", content);
    } else {
        println!("{}", "No output found for this agent.".dimmed());
    }
    Ok(())
}

/// Steer is not applicable in ACP mode (agents run to completion)
pub async fn steer(agent_id: &str, message: &str) -> Result<()> {
    println!("{} ACP agents run to completion — cannot steer mid-execution.", "⚠".yellow());
    println!("  To modify behavior, update goals.md and re-run.");
    Ok(())
}

/// Kill all — clean up state
pub async fn kill_all() -> Result<()> {
    let found = config::find_active_state()?;
    let (dir, mut state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;
    for a in &mut state.agents {
        if a.status == AgentStatus::Running {
            a.status = AgentStatus::Failed;
            a.finished_at = Some(Utc::now());
        }
    }
    config::save_state(&dir, &state)?;
    println!("{} Marked all agents as stopped", "✗".red());
    Ok(())
}

/// Kill a specific agent
pub async fn kill(agent_id: &str) -> Result<()> {
    let found = config::find_active_state()?;
    let (dir, mut state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;
    if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent_id) {
        a.status = AgentStatus::Failed;
        a.finished_at = Some(Utc::now());
    }
    config::save_state(&dir, &state)?;
    println!("{} Marked {} as stopped", "✗".red(), agent_id);
    Ok(())
}

/// Clean up worktrees and state
pub fn clean(project_dir: &Path) -> Result<()> {
    println!("Cleaning up worktrees and branches...");
    worktree::remove_all(project_dir)?;
    let state_path = project_dir.join(".subspace").join("state.json");
    if state_path.exists() {
        std::fs::remove_file(&state_path)?;
    }
    println!("{} Cleaned up", "✓".green());
    Ok(())
}

// ─── Prompt Builders ─────────────────────────────────────────

fn build_executor_prompt(proj: &Project, task: &planner::PlannedTask) -> String {
    format!(
        r#"# Subspace Executor Agent

You are an executor agent in a parallel AI coding swarm. You have ONE specific task to implement.
Your work is isolated in a git worktree — you cannot affect other agents.

## Project Stack: {}

## Your Task

{}

## Rules

1. Implement ONLY this task — do not go beyond scope
2. Commit your changes with message: `subspace: {}`
3. Run quality checks before committing:
{}
4. If you get stuck, commit what you have with a note in the commit message
5. Do NOT modify files outside your task scope
6. Append a brief summary to progress.md

## Important

You are working in a git worktree (isolated branch). Other agents are working on other tasks
simultaneously in their own worktrees. A merge agent will combine all branches afterward.
Focus on your task only."#,
        proj.stack,
        task.content,
        task.title,
        quality_checks(&proj.stack),
    )
}

fn build_merge_prompt(
    proj: &Project,
    base_branch: &str,
    completed_agents: &[String],
    branch_details: &str,
) -> String {
    let branch_list = completed_agents
        .iter()
        .map(|id| format!("subspace/{}", id))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"# Subspace Merge Agent

You are the merge coordinator for a parallel AI coding swarm. Multiple executor agents have
completed their tasks in separate git worktrees. Your job is to merge their work into `{}`.

## Project Stack: {}

## Branches to Merge

{}

## Merge Strategy

1. Review each branch's diff to understand what was changed
2. Merge branches one at a time, starting with the smallest diff
3. For each merge: `git merge subspace/<agent-id> --no-ff -m "subspace: merge <agent-id>"`
4. If there are conflicts:
   - Read both sides carefully
   - Understand what each agent intended
   - Resolve by keeping both changes where possible
   - If truly incompatible, prefer the more complete implementation
5. After all merges, run quality checks:
{}
6. Fix any issues that arise from the merge
7. Final commit: `subspace: merge complete [{}]`

## Rules

- Do NOT discard any agent's work unless it's genuinely broken
- If a merge conflict is complex, prefer the more complete implementation
- After merging, run the FULL test suite to ensure nothing broke
- Update progress.md with what was merged and any conflict resolutions

## Branch Details

{}"#,
        base_branch,
        proj.stack,
        completed_agents
            .iter()
            .enumerate()
            .map(|(i, id)| format!("{}. `subspace/{}`", i + 1, id))
            .collect::<Vec<_>>()
            .join("\n"),
        quality_checks(&proj.stack),
        branch_list,
        branch_details,
    )
}

fn quality_checks(stack: &str) -> String {
    let mut checks = Vec::new();
    if stack.contains("rust") {
        checks.extend(["   - `cargo check`", "   - `cargo test`", "   - `cargo clippy`"]);
    }
    if stack.contains("node") || stack.contains("typescript") {
        checks.extend(["   - `npm run typecheck` (if available)", "   - `npm test` (if available)"]);
    }
    if stack.contains("python") {
        checks.extend(["   - `pytest`", "   - `mypy` (if configured)"]);
    }
    if stack.contains("go") {
        checks.extend(["   - `go vet ./...`", "   - `go test ./...`"]);
    }
    if checks.is_empty() {
        checks.push("   - Run any available test/lint commands");
    }
    checks.join("\n")
}

fn detect_conflicts(state: &SwarmState) -> Vec<(String, Vec<String>)> {
    use std::collections::HashMap;
    let mut file_agents: HashMap<String, Vec<String>> = HashMap::new();

    for agent in &state.agents {
        if agent.role != AgentRole::Executor {
            continue;
        }
        for file in &agent.files_modified {
            file_agents
                .entry(file.clone())
                .or_default()
                .push(agent.id.clone());
        }
        if let Some(task) = &agent.task {
            for file in &task.files {
                file_agents
                    .entry(file.clone())
                    .or_default()
                    .push(agent.id.clone());
            }
        }
    }

    file_agents
        .into_iter()
        .filter(|(_, agents)| {
            let mut unique: Vec<_> = agents.clone();
            unique.sort();
            unique.dedup();
            unique.len() > 1
        })
        .map(|(file, mut agents)| {
            agents.sort();
            agents.dedup();
            (file, agents)
        })
        .collect()
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
