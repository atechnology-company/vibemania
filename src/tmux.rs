#[allow(dead_code)]

use anyhow::{bail, Context, Result};
use std::process::Command;

/// Check if tmux is available
pub fn check_tmux() -> Result<()> {
    let out = Command::new("tmux").arg("-V").output();
    match out {
        Ok(o) if o.status.success() => Ok(()),
        _ => bail!("tmux not found. Install it: apt install tmux / brew install tmux"),
    }
}

/// Check if a tmux session exists
pub fn session_exists(session: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", session])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create a new tmux session (detached)
pub fn new_session(session: &str, window_name: &str, command: &str, workdir: &str) -> Result<()> {
    let status = Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s", session,
            "-n", window_name,
            "-c", workdir,
            command,
        ])
        .status()
        .context("Failed to create tmux session")?;
    if !status.success() {
        bail!("tmux new-session failed");
    }
    Ok(())
}

/// Create a new window in an existing session
pub fn new_window(session: &str, window_name: &str, command: &str, workdir: &str) -> Result<()> {
    let status = Command::new("tmux")
        .args([
            "new-window",
            "-t", session,
            "-n", window_name,
            "-c", workdir,
            command,
        ])
        .status()
        .context("Failed to create tmux window")?;
    if !status.success() {
        bail!("tmux new-window failed");
    }
    Ok(())
}

/// Capture pane output (last N lines)
pub fn capture_pane(target: &str, lines: u32) -> Result<String> {
    let output = Command::new("tmux")
        .args([
            "capture-pane",
            "-t", target,
            "-p",
            "-S", &format!("-{}", lines),
        ])
        .output()
        .context("Failed to capture tmux pane")?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Send keys to a tmux target
pub fn send_keys(target: &str, keys: &str) -> Result<()> {
    let status = Command::new("tmux")
        .args(["send-keys", "-t", target, keys, "Enter"])
        .status()
        .context("Failed to send keys")?;
    if !status.success() {
        bail!("tmux send-keys failed for target {}", target);
    }
    Ok(())
}

/// Send Ctrl-C to a tmux target
pub fn send_interrupt(target: &str) -> Result<()> {
    let status = Command::new("tmux")
        .args(["send-keys", "-t", target, "C-c"])
        .status()
        .context("Failed to send interrupt")?;
    if !status.success() {
        bail!("tmux send-keys C-c failed for target {}", target);
    }
    Ok(())
}

/// Kill a tmux window
pub fn kill_window(target: &str) -> Result<()> {
    let _ = Command::new("tmux")
        .args(["kill-window", "-t", target])
        .status();
    Ok(())
}

/// Kill entire tmux session
pub fn kill_session(session: &str) -> Result<()> {
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", session])
        .status();
    Ok(())
}

/// List windows in a session
pub fn list_windows(session: &str) -> Result<Vec<String>> {
    let output = Command::new("tmux")
        .args([
            "list-windows",
            "-t", session,
            "-F", "#{window_name}",
        ])
        .output()
        .context("Failed to list tmux windows")?;
    let names: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();
    Ok(names)
}

/// Check if a pane is still running a process
pub fn pane_is_active(target: &str) -> bool {
    let output = Command::new("tmux")
        .args([
            "display-message",
            "-t", target,
            "-p", "#{pane_dead}",
        ])
        .output();
    match output {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            s != "1"
        }
        Err(_) => false,
    }
}

/// Build tmux target string: session:window
pub fn target(session: &str, window: &str) -> String {
    format!("{}:{}", session, window)
}
