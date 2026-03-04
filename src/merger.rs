#[allow(dead_code)]
use anyhow::Result;
use colored::*;

use crate::config::{self, AgentRole, AgentState, AgentStatus};
use crate::project::Project;
use crate::worktree;

const MERGER_PROMPT_TEMPLATE: &str = r#"# Subspace Merge Agent

You are the MERGE agent for Subspace, an AI coding agent swarm orchestrator.
Multiple executor agents have been working on independent tasks in separate git worktrees.
Your job is to merge their work back into the main branch cleanly.

## Project Stack: {{STACK}}

## Base Branch: {{BASE_BRANCH}}

## Agent Branches to Merge

{{BRANCH_DETAILS}}

## Your Job

1. Review the diffs from each branch
2. Merge each branch into the base branch, one at a time
3. If there are conflicts, resolve them intelligently:
   - Understand what each agent was trying to do
   - Keep both changes where possible
   - If truly conflicting, pick the better implementation
4. Run quality checks after each merge
5. Commit the final merged result

## Merge Order

Merge in this order (least files changed first to minimize conflicts):
{{MERGE_ORDER}}

## Quality Checks (based on stack)

{{QUALITY_CHECKS}}

## Rules

- Do NOT discard any agent's work unless it's truly broken
- If a merge conflict is complex, prefer the more complete implementation
- After all merges, run the full test suite
- Commit with message: `subspace: merge [branch-list]`
- If quality checks fail after merge, fix the issues
- Log what you did in progress.md
"#;

/// Build the merge prompt with all branch context
pub fn build_merge_prompt(proj: &Project, base_branch: &str, agent_ids: &[String]) -> Result<String> {
    let mut branch_details = String::new();
    let mut merge_order: Vec<(String, usize)> = Vec::new();

    for agent_id in agent_ids {
        let diff_stat = worktree::diff_from_base(&proj.dir, agent_id, base_branch)?;
        let commits = worktree::branch_log(&proj.dir, agent_id, base_branch)?;
        let full_diff = worktree::full_diff(&proj.dir, agent_id, base_branch)?;

        let file_count = diff_stat.lines().count();
        merge_order.push((agent_id.clone(), file_count));

        branch_details.push_str(&format!(
            "### Branch: subspace/{}\n\n#### Commits:\n```\n{}\n```\n\n#### Files Changed:\n```\n{}\n```\n\n#### Full Diff:\n```diff\n{}\n```\n\n---\n\n",
            agent_id, commits.trim(), diff_stat.trim(), full_diff.trim()
        ));
    }

    // Sort by file count (least first)
    merge_order.sort_by_key(|(_, count)| *count);
    let order_str = merge_order
        .iter()
        .enumerate()
        .map(|(i, (id, count))| format!("{}. `subspace/{}` ({} files)", i + 1, id, count))
        .collect::<Vec<_>>()
        .join("\n");

    let quality_checks = get_quality_checks(&proj.stack);

    let prompt = MERGER_PROMPT_TEMPLATE
        .replace("{{STACK}}", &proj.stack)
        .replace("{{BASE_BRANCH}}", base_branch)
        .replace("{{BRANCH_DETAILS}}", &branch_details)
        .replace("{{MERGE_ORDER}}", &order_str)
        .replace("{{QUALITY_CHECKS}}", &quality_checks);

    Ok(prompt)
}

/// Write merge prompt and return path
pub fn write_merge_prompt(proj: &Project, base_branch: &str, agent_ids: &[String]) -> Result<String> {
    let subspace_dir = proj.dir.join(".subspace");
    std::fs::create_dir_all(&subspace_dir)?;
    let path = subspace_dir.join("merger-prompt.md");
    let prompt = build_merge_prompt(proj, base_branch, agent_ids)?;
    std::fs::write(&path, &prompt)?;
    Ok(path.to_string_lossy().to_string())
}

/// Spawn the merge agent (uses Sonnet via claude --model)
pub fn spawn_merger(
    session: &str,
    proj: &Project,
    base_branch: &str,
    agent_ids: &[String],
) -> Result<String> {
    let prompt_file = write_merge_prompt(proj, base_branch, agent_ids)?;
    let merger_id = "merger";

    // Merger runs in the main repo dir (not a worktree)
    // Uses claude with sonnet model for cost efficiency
    let cmd = format!(
        "claude --dangerously-skip-permissions --model claude-sonnet-4-20250514 --print < '{}' 2>&1 | tee .subspace/merger.log; echo '\\n[SUBSPACE_AGENT_DONE]'",
        prompt_file
    );

    if crate::tmux::session_exists(session) {
        crate::tmux::new_window(session, merger_id, &cmd, &proj.dir.to_string_lossy())?;
    } else {
        crate::tmux::new_session(session, merger_id, &cmd, &proj.dir.to_string_lossy())?;
    }

    println!("  {} Merger agent spawned (Sonnet) in {}", "🔀".green(), session.cyan());

    Ok(merger_id.to_string())
}

/// Run the full merge flow
pub async fn run_merge(proj: &Project) -> Result<()> {
    let state = config::load_state(&proj.dir)?;
    let state = match state {
        Some(s) => s,
        None => {
            println!("{}", "No active swarm state found.".dimmed());
            return Ok(());
        }
    };

    let base_branch = worktree::current_branch(&proj.dir)?;

    // Find completed executor agents
    let completed_agents: Vec<String> = state
        .agents
        .iter()
        .filter(|a| a.role == AgentRole::Executor && a.status == AgentStatus::Completed)
        .map(|a| a.id.clone())
        .collect();

    if completed_agents.is_empty() {
        println!("{}", "No completed executor agents to merge.".yellow());
        return Ok(());
    }

    println!("{}", "═══ Subspace Merge ═══".bold());
    println!("Base branch: {}", base_branch.green());
    println!("Merging {} agent branch(es):", completed_agents.len());
    for id in &completed_agents {
        let diff = worktree::diff_from_base(&proj.dir, id, &base_branch)?;
        let file_count = diff.lines().count().saturating_sub(1); // minus summary line
        println!("  {} subspace/{} ({} files changed)", "→".cyan(), id, file_count);
    }
    println!();

    let session = &state.session_name;
    let merger_id = spawn_merger(session, proj, &base_branch, &completed_agents)?;

    // Update state
    let mut state = state;
    state.agents.push(AgentState {
        id: merger_id.clone(),
        role: AgentRole::Planner, // reuse role, it's the merge coordinator
        tmux_window: merger_id.clone(),
        status: AgentStatus::Running,
        started_at: chrono::Utc::now(),
        finished_at: None,
        task: Some(config::TaskInfo {
            title: "Merge all agent branches".to_string(),
            files: Vec::new(),
        }),
        files_modified: Vec::new(),
    });
    config::save_state(&proj.dir, &state)?;

    println!("Merger is running. Monitor with: {} swarm logs merger -f", "subspace".bold());
    Ok(())
}

fn get_quality_checks(stack: &str) -> String {
    let mut checks = Vec::new();
    if stack.contains("rust") {
        checks.push("cargo check");
        checks.push("cargo test");
        checks.push("cargo clippy");
    }
    if stack.contains("node") || stack.contains("typescript") {
        checks.push("npm run typecheck (if available)");
        checks.push("npm run lint (if available)");
        checks.push("npm test (if available)");
    }
    if stack.contains("python") {
        checks.push("pytest");
        checks.push("mypy (if configured)");
    }
    if stack.contains("go") {
        checks.push("go vet ./...");
        checks.push("go test ./...");
    }
    if checks.is_empty() {
        checks.push("Run any available test/lint commands");
    }
    checks.iter().map(|c| format!("- `{}`", c)).collect::<Vec<_>>().join("\n")
}
