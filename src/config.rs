use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub role: AgentRole,
    pub tmux_window: String,
    pub status: AgentStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub task: Option<TaskInfo>,
    pub files_modified: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    Planner,
    Executor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub title: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmState {
    pub session_name: String,
    pub project_dir: String,
    pub stack: String,
    pub agents: Vec<AgentState>,
    pub iteration: u32,
    pub conflicts: Vec<FileConflict>,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConflict {
    pub file: String,
    pub agents: Vec<String>,
}

impl SwarmState {
    pub fn new(session_name: &str, project_dir: &str, stack: &str) -> Self {
        Self {
            session_name: session_name.to_string(),
            project_dir: project_dir.to_string(),
            stack: stack.to_string(),
            agents: Vec::new(),
            iteration: 0,
            conflicts: Vec::new(),
            started_at: Utc::now(),
        }
    }
}

fn state_path(project_dir: &Path) -> PathBuf {
    project_dir.join(".subspace").join("state.json")
}

pub fn load_state(project_dir: &Path) -> Result<Option<SwarmState>> {
    let path = state_path(project_dir);
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let state: SwarmState = serde_json::from_str(&data)?;
    Ok(Some(state))
}

pub fn save_state(project_dir: &Path, state: &SwarmState) -> Result<()> {
    let path = state_path(project_dir);
    std::fs::create_dir_all(path.parent().unwrap())?;
    let data = serde_json::to_string_pretty(state)?;
    std::fs::write(&path, data)?;
    Ok(())
}

/// Find the most recent state file across known locations
pub fn find_active_state() -> Result<Option<(PathBuf, SwarmState)>> {
    // Check current directory first
    let cwd = std::env::current_dir()?;
    if let Some(state) = load_state(&cwd)? {
        return Ok(Some((cwd, state)));
    }

    // Check home directory for a global pointer
    let global_path = dirs_path().join("active_project.txt");
    if global_path.exists() {
        let dir = std::fs::read_to_string(&global_path)?;
        let dir = PathBuf::from(dir.trim());
        if let Some(state) = load_state(&dir)? {
            return Ok(Some((dir, state)));
        }
    }

    Ok(None)
}

pub fn set_active_project(project_dir: &Path) -> Result<()> {
    let global_dir = dirs_path();
    std::fs::create_dir_all(&global_dir)?;
    std::fs::write(
        global_dir.join("active_project.txt"),
        project_dir.to_string_lossy().as_bytes(),
    )?;
    Ok(())
}

fn dirs_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".subspace")
}
