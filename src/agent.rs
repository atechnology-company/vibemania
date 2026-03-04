#[allow(dead_code)]

use anyhow::Result;
use chrono::Utc;

use crate::config::{AgentRole, AgentState, AgentStatus, TaskInfo};
use crate::tmux;

/// Build the CLI command to run an AI agent
pub fn build_command(tool: &str, prompt_file: &str) -> String {
    match tool {
        "claude" => format!(
            "claude --dangerously-skip-permissions --print < '{}' 2>&1 | tee /tmp/subspace-agent-$$.log; echo '\\n[SUBSPACE_AGENT_DONE]'",
            prompt_file
        ),
        "amp" => format!(
            "cat '{}' | amp --dangerously-allow-all 2>&1 | tee /tmp/subspace-agent-$$.log; echo '\\n[SUBSPACE_AGENT_DONE]'",
            prompt_file
        ),
        _ => format!(
            "claude --dangerously-skip-permissions --print < '{}' 2>&1 | tee /tmp/subspace-agent-$$.log; echo '\\n[SUBSPACE_AGENT_DONE]'",
            prompt_file
        ),
    }
}

/// Spawn an agent in a tmux window
pub fn spawn(
    session: &str,
    agent_id: &str,
    role: AgentRole,
    tool: &str,
    prompt_file: &str,
    workdir: &str,
    task: Option<TaskInfo>,
) -> Result<AgentState> {
    let cmd = build_command(tool, prompt_file);

    if tmux::session_exists(session) {
        tmux::new_window(session, agent_id, &cmd, workdir)?;
    } else {
        tmux::new_session(session, agent_id, &cmd, workdir)?;
    }

    Ok(AgentState {
        id: agent_id.to_string(),
        role,
        tmux_window: agent_id.to_string(),
        status: AgentStatus::Running,
        started_at: Utc::now(),
        finished_at: None,
        task,
        files_modified: Vec::new(),
    })
}

/// Check if an agent is still running
pub fn is_running(session: &str, agent_id: &str) -> bool {
    let target = tmux::target(session, agent_id);
    tmux::pane_is_active(&target)
}

/// Check if agent output contains the completion marker
pub fn is_done(session: &str, agent_id: &str) -> Result<bool> {
    let target = tmux::target(session, agent_id);
    let output = tmux::capture_pane(&target, 50)?;
    Ok(output.contains("[SUBSPACE_AGENT_DONE]"))
}

/// Get recent output from an agent
pub fn get_output(session: &str, agent_id: &str, lines: u32) -> Result<String> {
    let target = tmux::target(session, agent_id);
    tmux::capture_pane(&target, lines)
}

/// Send a message/instruction to a running agent
pub fn steer(session: &str, agent_id: &str, message: &str) -> Result<()> {
    let target = tmux::target(session, agent_id);
    tmux::send_keys(&target, message)
}

/// Kill an agent
pub fn kill(session: &str, agent_id: &str) -> Result<()> {
    let target = tmux::target(session, agent_id);
    tmux::send_interrupt(&target)?;
    // Give it a moment then kill the window
    std::thread::sleep(std::time::Duration::from_millis(500));
    tmux::kill_window(&target)?;
    Ok(())
}
