//! On-device AI backend for lightweight classification/triage tasks.
//!
//! Falls back to a cheap heuristic if no platform model is available.
//! Enabled with --local flag; used automatically for triage when available.

pub mod apple;
pub mod windows;

use anyhow::Result;

/// Capability flag — checked at startup
#[derive(Debug, Clone, PartialEq)]
pub enum LocalBackend {
    Apple,   // macOS 15+ FoundationModels via ObjC FFI
    Windows, // Windows 11 Copilot+ via WinRT
    None,    // Unavailable — use heuristic fallback
}

/// Detect which on-device backend is available
pub fn detect() -> LocalBackend {
    if cfg!(target_os = "macos") && apple::is_available() {
        LocalBackend::Apple
    } else if cfg!(target_os = "windows") && windows::is_available() {
        LocalBackend::Windows
    } else {
        LocalBackend::None
    }
}

/// Lightweight prompt — runs on-device or falls back to heuristic
pub async fn run(backend: &LocalBackend, prompt: &str) -> Result<String> {
    match backend {
        LocalBackend::Apple => apple::run(prompt).await,
        LocalBackend::Windows => windows::run(prompt).await,
        LocalBackend::None => Ok(heuristic_fallback(prompt)),
    }
}

/// Simple heuristics when no on-device model exists
fn heuristic_fallback(prompt: &str) -> String {
    let p = prompt.to_lowercase();
    // Conflict triage
    if p.contains("conflict") || p.contains("overlap") {
        return "yes".to_string();
    }
    // Parallel task check
    if p.contains("parallel") || p.contains("independent") {
        return "yes".to_string();
    }
    // Default
    "unknown".to_string()
}

/// Triage: can these two tasks run in parallel (different files)?
pub async fn can_parallelize(backend: &LocalBackend, task_a: &str, task_b: &str) -> bool {
    let prompt = format!(
        "Answer only YES or NO. Can these two coding tasks be done in parallel without file conflicts?\nTask A: {}\nTask B: {}",
        task_a, task_b
    );
    run(backend, &prompt).await
        .map(|r| r.trim().to_uppercase().starts_with("YES"))
        .unwrap_or(true) // assume yes if uncertain
}

/// Triage: does this agent output indicate success?
pub async fn is_success(backend: &LocalBackend, output: &str) -> bool {
    let snippet = &output[..output.len().min(500)];
    let prompt = format!(
        "Answer only YES or NO. Did this agent output indicate successful task completion?\n{}",
        snippet
    );
    run(backend, &prompt).await
        .map(|r| r.trim().to_uppercase().starts_with("YES"))
        .unwrap_or(true)
}

/// Summarize agent output to 1 sentence for TUI log
pub async fn summarize(backend: &LocalBackend, output: &str) -> String {
    let snippet = &output[..output.len().min(800)];
    let prompt = format!(
        "Summarize in ONE short sentence (max 80 chars) what this agent did:\n{}",
        snippet
    );
    run(backend, &prompt).await
        .unwrap_or_else(|_| "completed task".to_string())
}
