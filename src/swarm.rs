//! Swarm orchestrator — ACP agents, worktree isolation, Haiku merger.

use anyhow::Result;
use chrono::Utc;
use colored::*;
use std::path::{Path, PathBuf};

use crate::acp;
use crate::config::{self, AgentRole, AgentState, AgentStatus, SwarmState, TaskInfo};
use crate::planner;
use crate::project::{self, Project};
use crate::worktree;

/// Run the full plan/execute/merge loop
pub async fn run_loop(proj: &Project, goal: Option<&str>, max_iter: u32, parallel: u32) -> Result<()> {
    acp::find_acp_binary()?;

    let session_name = project::session_name(&proj.dir);
    let base_branch = worktree::current_branch(&proj.dir)?;

    println!("{}", "═══════════════════════════════════════════".bold());
    println!("  {} {}", "Subspace".bold().cyan(), "— Agent Swarm");
    println!("{}", "═══════════════════════════════════════════".bold());
    println!("Goal:       {}", goal.unwrap_or("autonomous (discover → audit → complete)").white().bold());
    println!("Project:    {}", proj.dir.display().to_string().cyan());
    println!("Stack:      {}", proj.stack.yellow());
    println!("Base:       {}", base_branch.green());
    println!("Agents:     {} parallel", parallel);
    println!("Executors:  {}", "Sonnet (ACP)".blue());
    println!("Merger:     {}", "Haiku (ACP)".green());
    println!("Permissions: {}", "auto-approve".red());
    println!();

    let mut state = SwarmState::new(&session_name, &proj.dir.to_string_lossy(), &proj.stack);
    config::set_active_project(&proj.dir)?;

    let subspace_dir = proj.dir.join(".subspace");
    std::fs::create_dir_all(&subspace_dir)?;

    for i in 1..=max_iter {
        println!();
        println!("{}", format!("═══ Iteration {} / {} ═══", i, max_iter).bold());
        state.iteration = i;

        // ── Phase A: Plan ──
        println!("{}", "  🧠 Planning...".dimmed());
        let proj = project::load(&proj.dir)?;
        let planner_prompt = planner::build_prompt(&proj, goal, i);
        std::fs::write(subspace_dir.join("planner-prompt.md"), &planner_prompt)?;

        let plan = acp::run_prompt(&proj.dir, &planner_prompt).await?;
        std::fs::write(subspace_dir.join("plan.md"), &plan.text)?;

        state.agents.retain(|a| a.role != AgentRole::Planner);
        state.agents.push(AgentState {
            id: "planner".into(),
            role: AgentRole::Planner,
            tmux_window: String::new(),
            status: AgentStatus::Completed,
            started_at: Utc::now(),
            finished_at: Some(Utc::now()),
            task: None,
            files_modified: plan.files_modified.clone(),
        });
        config::save_state(&proj.dir, &state)?;

        if planner::is_complete(&plan.text) {
            println!();
            println!("{}", "  ✅ All goals achieved!".green().bold());
            return Ok(());
        }

        let tasks = planner::parse_tasks(&plan.text);
        let task_count = tasks.len().min(parallel as usize);
        println!("  {} task(s), running {} in parallel:", tasks.len(), task_count);
        for t in tasks.iter().take(task_count) {
            println!("    {} {}", "→".cyan(), t.title);
        }

        // ── Phase B: Execute (parallel, worktree-isolated) ──
        println!();
        println!("{}", "  🔨 Executing...".dimmed());

        let mut handles = Vec::new();
        let mut agent_ids = Vec::new();

        for task in tasks.iter().take(task_count) {
            let agent_id = format!("executor-{}", task.id);
            let wt_dir = worktree::create(&proj.dir, &agent_id, &base_branch)?;

            let prompt = format!(
                r#"# Subspace Executor

You are an executor in a parallel AI swarm. Implement ONLY this task.
You're in an isolated git worktree — other agents can't see your changes.

## Stack: {stack}

## Task
{content}

## Rules
- Implement ONLY this task, nothing else
- Commit with message: `subspace: {title}`
- Run quality checks: {checks}
- Update progress.md with what you did
- Do NOT touch files outside your task scope"#,
                stack = proj.stack,
                content = task.content,
                title = task.title,
                checks = quality_checks(&proj.stack),
            );

            std::fs::write(
                subspace_dir.join(format!("executor-{}-prompt.md", task.id)),
                &prompt,
            )?;

            state.agents.push(AgentState {
                id: agent_id.clone(),
                role: AgentRole::Executor,
                tmux_window: String::new(),
                status: AgentStatus::Running,
                started_at: Utc::now(),
                finished_at: None,
                task: Some(TaskInfo { title: task.title.clone(), files: task.files.clone() }),
                files_modified: Vec::new(),
            });

            agent_ids.push(agent_id.clone());

            let wt_path = wt_dir.clone();
            let handle = tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                let result = rt.block_on(acp::run_prompt(&wt_path, &prompt))?;
                Ok::<(String, acp::SessionOutput), anyhow::Error>((agent_id, result))
            });
            handles.push(handle);
        }
        config::save_state(&proj.dir, &state)?;

        // Wait for all executors
        let mut completed = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok((id, output))) => {
                    println!("    {} {} — {} tools, {} files",
                        "✓".green(), id.bold(),
                        output.tool_calls.len(), output.files_modified.len());
                    std::fs::write(subspace_dir.join(format!("{}-output.md", id)), &output.text)?;
                    if let Some(a) = state.agents.iter_mut().find(|a| a.id == id) {
                        a.status = AgentStatus::Completed;
                        a.finished_at = Some(Utc::now());
                        a.files_modified = output.files_modified;
                    }
                    completed.push(id);
                }
                Ok(Err(e)) => println!("    {} agent error: {}", "✗".red(), e),
                Err(e) => println!("    {} agent panic: {}", "✗".red(), e),
            }
        }
        config::save_state(&proj.dir, &state)?;

        if completed.is_empty() {
            println!("    {} No agents completed, retrying...", "⚠".yellow());
            continue;
        }

        // ── Phase C: Merge (Haiku — cheap + fast) ──
        println!();
        println!("{}", "  🔀 Merging (Haiku)...".dimmed());

        let merge_prompt = build_merge_prompt(&proj, &base_branch, &completed, &subspace_dir)?;
        std::fs::write(subspace_dir.join("merger-prompt.md"), &merge_prompt)?;

        let merge_result = acp::run_prompt(&proj.dir, &merge_prompt).await?;
        std::fs::write(subspace_dir.join("merger-output.md"), &merge_result.text)?;

        println!("    {} Merge done ({} tools)", "✓".green(), merge_result.tool_calls.len());

        // Detect conflicts
        let conflicts = detect_conflicts(&state);
        if !conflicts.is_empty() {
            println!("    {} {} conflict(s):", "⚠".yellow(), conflicts.len());
            for (f, agents) in &conflicts {
                println!("      {} — {}", f.yellow(), agents.join(", "));
            }
        }
        state.conflicts = conflicts.into_iter()
            .map(|(file, agents)| config::FileConflict { file, agents })
            .collect();
        config::save_state(&proj.dir, &state)?;
    }

    println!("\n{} Max iterations reached. Check progress.md", "⚠".yellow());
    Ok(())
}

/// Launch agents (no loop — plan once, execute, wait for merge command)
pub async fn launch(proj: &Project, goal: Option<&str>, agent_count: u32) -> Result<()> {
    acp::find_acp_binary()?;
    let base_branch = worktree::current_branch(&proj.dir)?;
    let session_name = project::session_name(&proj.dir);
    let subspace_dir = proj.dir.join(".subspace");
    std::fs::create_dir_all(&subspace_dir)?;

    println!("{} Planning for: {}", "🧠", goal.unwrap_or("autonomous discovery").bold());
    let planner_prompt = planner::build_prompt(proj, goal, 1);
    let plan = acp::run_prompt(&proj.dir, &planner_prompt).await?;
    let tasks = planner::parse_tasks(&plan.text);
    let n = tasks.len().min(agent_count as usize);

    let mut state = SwarmState::new(&session_name, &proj.dir.to_string_lossy(), &proj.stack);
    state.iteration = 1;
    config::set_active_project(&proj.dir)?;

    println!("Spawning {} agent(s):", n);
    let mut handles = Vec::new();

    for task in tasks.iter().take(n) {
        let agent_id = format!("executor-{}", task.id);
        let wt_dir = worktree::create(&proj.dir, &agent_id, &base_branch)?;
        let prompt = format!(
            "# Executor\nStack: {}\n## Task\n{}\n\nCommit as `subspace: {}`. Run: {}",
            proj.stack, task.content, task.title, quality_checks(&proj.stack),
        );
        println!("  {} {} — {}", "→".green(), agent_id.bold(), task.title.cyan());

        state.agents.push(AgentState {
            id: agent_id.clone(), role: AgentRole::Executor, tmux_window: String::new(),
            status: AgentStatus::Running, started_at: Utc::now(), finished_at: None,
            task: Some(TaskInfo { title: task.title.clone(), files: task.files.clone() }),
            files_modified: Vec::new(),
        });

        let wt_path = wt_dir.clone();
        handles.push(tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
            let r = rt.block_on(acp::run_prompt(&wt_path, &prompt))?;
            Ok::<(String, acp::SessionOutput), anyhow::Error>((agent_id, r))
        }));
    }
    config::save_state(&proj.dir, &state)?;

    println!("\nWaiting...");
    for h in handles {
        match h.await {
            Ok(Ok((id, out))) => {
                println!("  {} {}", "✓".green(), id);
                if let Some(a) = state.agents.iter_mut().find(|a| a.id == id) {
                    a.status = AgentStatus::Completed;
                    a.finished_at = Some(Utc::now());
                    a.files_modified = out.files_modified;
                }
            }
            Ok(Err(e)) => println!("  {} {}", "✗".red(), e),
            Err(e) => println!("  {} {}", "✗".red(), e),
        }
    }
    config::save_state(&proj.dir, &state)?;
    println!("\nRun {} to merge.", "subspace swarm merge".bold());
    Ok(())
}

pub async fn status() -> Result<()> {
    let found = config::find_active_state()?;
    match found {
        None => println!("{}", "No active swarm.".dimmed()),
        Some((_, state)) => {
            println!("{}", "═══ Subspace Status ═══".bold());
            println!("Session: {} | Stack: {} | Iter: {}",
                state.session_name.green(), state.stack.yellow(), state.iteration);
            for a in &state.agents {
                let s = match a.status {
                    AgentStatus::Running => "●".green(),
                    AgentStatus::Completed => "✓".blue(),
                    AgentStatus::Failed => "✗".red(),
                    AgentStatus::Pending => "◌".dimmed(),
                };
                let r = if a.role == AgentRole::Planner { "🧠" } else { "🔨" };
                println!("  {} {} {} {}", r, s, a.id.bold(),
                    a.task.as_ref().map(|t| t.title.as_str()).unwrap_or(""));
                if !a.files_modified.is_empty() {
                    println!("      files: {}", a.files_modified.join(", ").dimmed());
                }
            }
        }
    }
    Ok(())
}

pub async fn logs(agent_id: &str) -> Result<()> {
    let found = config::find_active_state()?;
    let (dir, _) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;
    let p = PathBuf::from(&dir).join(".subspace").join(format!("{}-output.md", agent_id));
    if p.exists() {
        println!("{}", std::fs::read_to_string(&p)?);
    } else {
        println!("{}", "No output found.".dimmed());
    }
    Ok(())
}

pub async fn kill_all() -> Result<()> {
    let found = config::find_active_state()?;
    let (dir, mut state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;
    for a in &mut state.agents {
        if a.status == AgentStatus::Running { a.status = AgentStatus::Failed; a.finished_at = Some(Utc::now()); }
    }
    config::save_state(&dir, &state)?;
    println!("{} Stopped all agents", "✗".red());
    Ok(())
}

pub async fn kill(agent_id: &str) -> Result<()> {
    let found = config::find_active_state()?;
    let (dir, mut state) = found.ok_or_else(|| anyhow::anyhow!("No active swarm"))?;
    if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent_id) {
        a.status = AgentStatus::Failed; a.finished_at = Some(Utc::now());
    }
    config::save_state(&dir, &state)?;
    println!("{} Stopped {}", "✗".red(), agent_id);
    Ok(())
}

pub fn clean(project_dir: &Path) -> Result<()> {
    worktree::remove_all(project_dir)?;
    let p = project_dir.join(".subspace").join("state.json");
    if p.exists() { std::fs::remove_file(&p)?; }
    println!("{} Cleaned", "✓".green());
    Ok(())
}

// ─── Helpers ───

fn build_merge_prompt(
    proj: &Project, base: &str, agents: &[String], subspace_dir: &Path,
) -> Result<String> {
    let mut details = String::new();
    for id in agents {
        let diff = worktree::diff_from_base(&proj.dir, id, base).unwrap_or_default();
        let full = worktree::full_diff(&proj.dir, id, base).unwrap_or_default();
        let log = worktree::branch_log(&proj.dir, id, base).unwrap_or_default();
        let out = std::fs::read_to_string(subspace_dir.join(format!("{}-output.md", id)))
            .unwrap_or_default();

        details.push_str(&format!(
            "### subspace/{id}\nSummary: {summary}\nCommits:\n```\n{log}\n```\nChanged:\n```\n{diff}\n```\nDiff:\n```diff\n{full_diff}\n```\n---\n\n",
            id = id,
            summary = truncate(&out, 1500),
            log = log.trim(),
            diff = diff.trim(),
            full_diff = truncate(&full, 6000),
        ));
    }

    Ok(format!(
        r#"# Subspace Merge Agent (Haiku)

Merge these agent branches into `{base}`. Each ran independently in a worktree.

## Branches
{list}

## Instructions
1. Merge each branch: `git merge subspace/<id> --no-ff`
2. Resolve conflicts (keep both changes where possible)
3. Run quality checks: {checks}
4. Commit: `subspace: merge complete`

## Branch Details
{details}"#,
        base = base,
        list = agents.iter().map(|id| format!("- subspace/{}", id)).collect::<Vec<_>>().join("\n"),
        checks = quality_checks(&proj.stack),
        details = details,
    ))
}

fn quality_checks(stack: &str) -> String {
    let mut c = Vec::new();
    if stack.contains("rust") { c.extend(["cargo check", "cargo test"]); }
    if stack.contains("node") || stack.contains("typescript") { c.extend(["npm test"]); }
    if stack.contains("python") { c.push("pytest"); }
    if stack.contains("go") { c.push("go test ./..."); }
    if c.is_empty() { c.push("run available tests"); }
    c.join(", ")
}

fn detect_conflicts(state: &SwarmState) -> Vec<(String, Vec<String>)> {
    use std::collections::HashMap;
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    for a in &state.agents {
        if a.role != AgentRole::Executor { continue; }
        let files: Vec<_> = a.files_modified.iter()
            .chain(a.task.iter().flat_map(|t| &t.files))
            .collect();
        for f in files {
            m.entry(f.clone()).or_default().push(a.id.clone());
        }
    }
    m.into_iter()
        .map(|(f, mut a)| { a.sort(); a.dedup(); (f, a) })
        .filter(|(_, a)| a.len() > 1)
        .collect()
}

/// Merge completed agent branches
pub async fn merge(project_dir: &std::path::Path) -> Result<()> {
    let state = config::load_state(project_dir)?;
    let state = match state {
        Some(s) => s,
        None => { println!("No active swarm."); return Ok(()); }
    };

    let base = worktree::current_branch(project_dir)?;
    let completed: Vec<String> = state.agents.iter()
        .filter(|a| a.role == AgentRole::Executor && a.status == AgentStatus::Completed)
        .map(|a| a.id.clone()).collect();

    if completed.is_empty() { println!("No completed agents."); return Ok(()); }

    let subspace_dir = project_dir.join(".subspace");
    let prompt = build_merge_prompt(
        &crate::project::load_or_init(project_dir)?,
        &base, &completed, &subspace_dir,
    )?;

    println!("Merging {} branch(es) with Haiku...", completed.len());
    let r = acp::run_prompt(project_dir, &prompt).await?;
    std::fs::write(subspace_dir.join("merger-output.md"), &r.text)?;
    println!("{} Merge done", "✓".green());
    Ok(())
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
