use anyhow::Result;

use crate::agent;
use crate::config::{AgentRole, TaskInfo};
use crate::planner::PlannedTask;
use crate::project::Project;

const EXECUTOR_TEMPLATE: &str = include_str!("../prompts/executor.md");

/// Build the executor prompt for a specific task
pub fn build_prompt(proj: &Project, task: &PlannedTask) -> String {
    let prompt = EXECUTOR_TEMPLATE.replace("{{STACK}}", &proj.stack);
    format!(
        "{}\n\n## The Plan To Execute\n\n{}",
        prompt, task.content
    )
}

/// Write executor prompt to file and return path
pub fn write_prompt(proj: &Project, task: &PlannedTask) -> Result<String> {
    let subspace_dir = proj.dir.join(".subspace");
    std::fs::create_dir_all(&subspace_dir)?;
    let path = subspace_dir.join(format!("executor-prompt-{}.md", task.id));
    let prompt = build_prompt(proj, task);
    std::fs::write(&path, &prompt)?;
    Ok(path.to_string_lossy().to_string())
}

/// Spawn an executor agent for a task
pub fn spawn_executor(
    session: &str,
    proj: &Project,
    task: &PlannedTask,
    tool: &str,
) -> Result<String> {
    let prompt_file = write_prompt(proj, task)?;
    let agent_id = format!("executor-{}", task.id);

    let task_info = TaskInfo {
        title: task.title.clone(),
        files: task.files.clone(),
    };

    agent::spawn(
        session,
        &agent_id,
        AgentRole::Executor,
        tool,
        &prompt_file,
        &proj.dir.to_string_lossy(),
        Some(task_info),
    )?;

    Ok(agent_id)
}
