use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Create a git worktree for an agent
pub fn create(repo_dir: &Path, agent_id: &str, base_branch: &str) -> Result<PathBuf> {
    let worktree_dir = worktree_path(repo_dir, agent_id);
    let branch_name = format!("subspace/{}", agent_id);

    // Create branch from base
    let status = Command::new("git")
        .args(["branch", &branch_name, base_branch])
        .current_dir(repo_dir)
        .status()
        .context("Failed to create branch")?;

    if !status.success() {
        // Branch might already exist, try to reset it
        let _ = Command::new("git")
            .args(["branch", "-D", &branch_name])
            .current_dir(repo_dir)
            .status();
        Command::new("git")
            .args(["branch", &branch_name, base_branch])
            .current_dir(repo_dir)
            .status()
            .context("Failed to create branch")?;
    }

    // Remove existing worktree if present
    if worktree_dir.exists() {
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force", &worktree_dir.to_string_lossy()])
            .current_dir(repo_dir)
            .status();
    }

    // Create worktree
    let status = Command::new("git")
        .args([
            "worktree",
            "add",
            &worktree_dir.to_string_lossy(),
            &branch_name,
        ])
        .current_dir(repo_dir)
        .status()
        .context("Failed to create worktree")?;

    if !status.success() {
        bail!("git worktree add failed for {}", agent_id);
    }

    Ok(worktree_dir)
}

/// Remove a worktree
pub fn remove(repo_dir: &Path, agent_id: &str) -> Result<()> {
    let worktree_dir = worktree_path(repo_dir, agent_id);
    let branch_name = format!("subspace/{}", agent_id);

    let _ = Command::new("git")
        .args(["worktree", "remove", "--force", &worktree_dir.to_string_lossy()])
        .current_dir(repo_dir)
        .status();

    let _ = Command::new("git")
        .args(["branch", "-D", &branch_name])
        .current_dir(repo_dir)
        .status();

    Ok(())
}

/// Remove all subspace worktrees
pub fn remove_all(repo_dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_dir)
        .output()
        .context("Failed to list worktrees")?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.starts_with("worktree ") {
            let path = line.trim_start_matches("worktree ");
            if path.contains(".subspace/worktrees/") {
                let _ = Command::new("git")
                    .args(["worktree", "remove", "--force", path])
                    .current_dir(repo_dir)
                    .status();
            }
        }
    }

    // Clean up subspace branches
    let output = Command::new("git")
        .args(["branch", "--list", "subspace/*"])
        .current_dir(repo_dir)
        .output()?;

    let branches = String::from_utf8_lossy(&output.stdout);
    for branch in branches.lines() {
        let branch = branch.trim().trim_start_matches("* ");
        if !branch.is_empty() {
            let _ = Command::new("git")
                .args(["branch", "-D", branch])
                .current_dir(repo_dir)
                .status();
        }
    }

    Ok(())
}

/// Get the current branch name
pub fn current_branch(repo_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo_dir)
        .output()
        .context("Failed to get current branch")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get diff between agent branch and base
pub fn diff_from_base(repo_dir: &Path, agent_id: &str, base_branch: &str) -> Result<String> {
    let branch_name = format!("subspace/{}", agent_id);
    let output = Command::new("git")
        .args(["diff", base_branch, &branch_name, "--stat"])
        .current_dir(repo_dir)
        .output()
        .context("Failed to get diff")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get full diff for merge context
pub fn full_diff(repo_dir: &Path, agent_id: &str, base_branch: &str) -> Result<String> {
    let branch_name = format!("subspace/{}", agent_id);
    let output = Command::new("git")
        .args(["diff", base_branch, &branch_name])
        .current_dir(repo_dir)
        .output()
        .context("Failed to get full diff")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get commit log for an agent's branch
pub fn branch_log(repo_dir: &Path, agent_id: &str, base_branch: &str) -> Result<String> {
    let branch_name = format!("subspace/{}", agent_id);
    let range = format!("{}..{}", base_branch, branch_name);
    let output = Command::new("git")
        .args(["log", "--oneline", &range])
        .current_dir(repo_dir)
        .output()
        .context("Failed to get branch log")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// List all subspace branches with their worktree paths
pub fn list(repo_dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut result = Vec::new();
    let output = Command::new("git")
        .args(["branch", "--list", "subspace/*"])
        .current_dir(repo_dir)
        .output()?;

    let branches = String::from_utf8_lossy(&output.stdout);
    for branch in branches.lines() {
        let branch = branch.trim().trim_start_matches("* ").to_string();
        if !branch.is_empty() {
            let agent_id = branch.trim_start_matches("subspace/").to_string();
            let wt_path = worktree_path(repo_dir, &agent_id);
            result.push((agent_id, wt_path));
        }
    }

    Ok(result)
}

fn worktree_path(repo_dir: &Path, agent_id: &str) -> PathBuf {
    repo_dir.join(".subspace").join("worktrees").join(agent_id)
}
