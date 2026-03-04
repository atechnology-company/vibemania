//! Dream mode — creative feature invention.
//!
//! Flow per cycle:
//!   1. Dreamer (Opus) imagines features
//!   2. Approval gate — user picks which to build (CLI prompt or TUI picker)
//!   3. Builders (Sonnet × N) implement approved features in parallel worktrees
//!   4. Merger (Haiku) merges feature branches
//!   5. Repeat

use anyhow::Result;
use chrono::Utc;
use colored::*;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::acp;
use crate::config::{self, AgentRole, AgentState, AgentStatus, SwarmState, TaskInfo};
use crate::local::{self, LocalBackend};
use crate::project::Project;
use crate::{tui_agent, tui_log, tui_phase};
use crate::tui::{AgentRole as TuiRole, AgentStatus as TuiStatus, AgentView, LogLevel, Phase, SharedTuiState, ApprovalRequest};
use crate::worktree;

const DREAMS_FILE: &str = "DREAMS.md";

/// Run dream mode — infinite creative loop with user approval gate
pub async fn run(
    proj: &Project,
    parallel: u32,
    tui_state: Option<SharedTuiState>,
    local_backend: &LocalBackend,
    auto_approve: bool,
) -> Result<()> {
    acp::find_acp_binary()?;
    let base_branch = worktree::current_branch(&proj.dir)?;
    let session_name = crate::project::session_name(&proj.dir);

    if tui_state.is_none() {
        println!("{}", "╔═══════════════════════════════════════════╗".magenta().bold());
        println!("{}", "║  ✨  SUBSPACE DREAM MODE  ✨               ║".magenta().bold());
        println!("{}", "╚═══════════════════════════════════════════╝".magenta().bold());
        println!("Project: {}", proj.dir.display().to_string().cyan());
        println!("Stack:   {}", proj.stack.yellow());
        println!("Model:   {}", "Opus (dreamer) + Sonnet (builders)".magenta());
        println!("Local:   {}", match local_backend {
            LocalBackend::Apple => "Apple Intelligence ✓".green().to_string(),
            LocalBackend::Windows => "Windows AI ✓".green().to_string(),
            LocalBackend::None => "none (heuristic fallback)".dimmed().to_string(),
        });
        println!("Approve: {}", if auto_approve {
            "auto (--auto-approve)".yellow().to_string()
        } else {
            "manual (you pick features)".cyan().to_string()
        });
        println!();
        println!("{}", "Press Ctrl+C to wake up.".dimmed());
        println!();
    }

    tui_phase!(tui_state, Phase::Planning);

    let mut iteration = 0u32;
    loop {
        iteration += 1;

        if tui_state.is_none() {
            println!("{}", format!("✨ Dream cycle {} ✨", iteration).magenta().bold());
        }
        tui_log!(tui_state, "dreamer", LogLevel::Info, "Dream cycle {} — imagining...", iteration);

        // ── Phase 1: Dreamer (Opus) ──────────────────────────────────────────
        let dreams_content = load_dreams(&proj.dir);
        let dreamer_prompt = build_dreamer_prompt(proj, &dreams_content, iteration);

        let subspace_dir = proj.dir.join(".subspace");
        std::fs::create_dir_all(&subspace_dir)?;
        std::fs::write(subspace_dir.join("dreamer-prompt.md"), &dreamer_prompt)?;

        tui_agent!(tui_state, AgentView {
            id: "dreamer".into(), role: TuiRole::Planner,
            status: TuiStatus::Running, current_tool: Some("imagining...".into()),
            files_touched: 0, tool_count: 0,
        });

        let dream_result = acp::run_prompt(&proj.dir, &dreamer_prompt).await?;
        std::fs::write(subspace_dir.join("dream-output.md"), &dream_result.text)?;

        tui_agent!(tui_state, AgentView {
            id: "dreamer".into(), role: TuiRole::Planner, status: TuiStatus::Done,
            current_tool: None, files_touched: 0, tool_count: dream_result.tool_calls.len() as u32,
        });

        let all_features = parse_dream_features(&dream_result.text);

        if all_features.is_empty() {
            tui_log!(tui_state, "dreamer", LogLevel::Warning, "No buildable features this cycle");
            if tui_state.is_none() { println!("  No buildable features dreamed, trying again..."); }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            continue;
        }

        tui_log!(tui_state, "dreamer", LogLevel::Success,
            "Dreamed {} features — waiting for approval", all_features.len());

        // ── Phase 2: Approval gate ───────────────────────────────────────────
        let approved = if auto_approve {
            // Auto: take up to `parallel` highest wow-factor features
            all_features[..all_features.len().min(parallel as usize)].to_vec()
        } else if tui_state.is_some() {
            // TUI: send approval request, block until user responds
            tui_approval_gate(&tui_state, &all_features, parallel).await?
        } else {
            // CLI: interactive prompt
            cli_approval_gate(&all_features, parallel)?
        };

        if approved.is_empty() {
            tui_log!(tui_state, "dreamer", LogLevel::Warning, "All features rejected — dreaming again");
            if tui_state.is_none() { println!("  {} Skipping this cycle.\n", "→".dimmed()); }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        tui_log!(tui_state, "dreamer", LogLevel::Info,
            "Approved {} feature(s) — building", approved.len());
        if tui_state.is_none() {
            println!("  Building {} approved feature(s):", approved.len());
            for f in &approved {
                println!("    {} {} (wow {}/10)", "→".magenta(), f.title.bold(), f.wow_factor);
            }
            println!();
        }

        // ── Phase 3: Local AI triage (parallelism check) ──────────────────────
        if approved.len() > 1 {
            for i in 0..approved.len() {
                for j in (i+1)..approved.len() {
                    let ok = local::can_parallelize(
                        local_backend, &approved[i].title, &approved[j].title
                    ).await;
                    if !ok {
                        tui_log!(tui_state, "local", LogLevel::Warning,
                            "⚠ {} and {} may conflict — will serialize", approved[i].title, approved[j].title);
                    }
                }
            }
        }

        // ── Phase 4: Builders (Sonnet × N) ───────────────────────────────────
        tui_phase!(tui_state, Phase::Executing);

        let mut state = SwarmState::new(&session_name, &proj.dir.to_string_lossy(), &proj.stack);
        state.iteration = iteration;
        config::set_active_project(&proj.dir)?;

        let mut handles = Vec::new();

        for (i, feature) in approved.iter().enumerate() {
            let agent_id = format!("builder-{}", i + 1);
            let wt_dir = worktree::create(&proj.dir, &agent_id, &base_branch)?;

            let prompt = build_builder_prompt(proj, feature);
            std::fs::write(
                subspace_dir.join(format!("builder-{}-prompt.md", i + 1)),
                &prompt,
            )?;

            state.agents.push(AgentState {
                id: agent_id.clone(),
                role: AgentRole::Executor,
                tmux_window: String::new(),
                status: AgentStatus::Running,
                started_at: Utc::now(),
                finished_at: None,
                task: Some(TaskInfo {
                    title: feature.title.clone(),
                    files: feature.files.clone(),
                }),
                files_modified: Vec::new(),
            });

            tui_agent!(tui_state, AgentView {
                id: agent_id.clone(), role: TuiRole::Executor, status: TuiStatus::Running,
                current_tool: Some(feature.title.chars().take(20).collect()),
                files_touched: 0, tool_count: 0,
            });
            tui_log!(tui_state, &agent_id, LogLevel::Info, "building: {}", feature.title);

            let wt_path = wt_dir.clone();
            handles.push(tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
                let r = rt.block_on(acp::run_prompt(&wt_path, &prompt))?;
                Ok::<(String, crate::acp::SessionOutput), anyhow::Error>((agent_id, r))
            }));
        }
        config::save_state(&proj.dir, &state)?;

        // Wait for builders
        let mut built = Vec::new();
        for h in handles {
            match h.await {
                Ok(Ok((id, out))) => {
                    // Use local AI to summarize output for TUI log
                    let summary = local::summarize(local_backend, &out.text).await;
                    tui_agent!(tui_state, AgentView {
                        id: id.clone(), role: TuiRole::Executor, status: TuiStatus::Done,
                        current_tool: None,
                        files_touched: out.files_modified.len() as u32,
                        tool_count: out.tool_calls.len() as u32,
                    });
                    tui_log!(tui_state, &id, LogLevel::Success, "{}", summary);
                    if tui_state.is_none() {
                        println!("  {} {} — {}", "✓".green(), id, summary);
                    }
                    std::fs::write(subspace_dir.join(format!("{}-output.md", id)), &out.text)?;
                    built.push(id);
                }
                Ok(Err(e)) => {
                    tui_log!(tui_state, "builder", LogLevel::Error, "failed: {}", e);
                    if tui_state.is_none() { println!("  {} failed: {}", "✗".red(), e); }
                }
                Err(e) => {
                    tui_log!(tui_state, "builder", LogLevel::Error, "panic: {}", e);
                }
            }
        }

        if built.is_empty() {
            if tui_state.is_none() { println!("  No builders succeeded, continuing..."); }
            continue;
        }

        // ── Phase 5: Merge (Haiku) ────────────────────────────────────────────
        tui_phase!(tui_state, Phase::Merging);
        tui_log!(tui_state, "merger", LogLevel::Info, "merging {} built features", built.len());

        let merge_prompt = build_dream_merge_prompt(proj, &base_branch, &built, &subspace_dir)?;
        let merge_result = acp::run_prompt(&proj.dir, &merge_prompt).await?;
        std::fs::write(subspace_dir.join("dream-merger-output.md"), &merge_result.text)?;

        tui_log!(tui_state, "merger", LogLevel::Success, "merged — next dream cycle");
        if tui_state.is_none() {
            println!("  {} Merged. Dreaming again...\n", "✨".to_string().magenta());
        }

        append_to_dreams(&proj.dir, &approved, iteration)?;
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

// ─── Approval gate: CLI ───────────────────────────────────────────────────

fn cli_approval_gate(features: &[DreamFeature], parallel: u32) -> Result<Vec<DreamFeature>> {
    println!();
    println!("{}", "═══ DREAMER RESULTS — Choose features to build ═══".magenta().bold());
    println!("{}", "Enter numbers separated by commas, 'all', or 'skip'".dimmed());
    println!();

    for (i, f) in features.iter().enumerate() {
        let bar = "█".repeat(f.wow_factor as usize);
        let wow_color = if f.wow_factor >= 8 { bar.green() }
            else if f.wow_factor >= 6 { bar.yellow() }
            else { bar.dimmed() };
        println!("  {}  {} {}/10", format!("[{}]", i + 1).cyan().bold(), wow_color, f.wow_factor);
        println!("      {}", f.title.bold().white());
        println!("      {}", f.description.chars().take(120).collect::<String>().dimmed());
        if !f.files.is_empty() {
            println!("      Files: {}", f.files.join(", ").dimmed());
        }
        println!();
    }

    let max = features.len().min(parallel as usize);
    print!("  {} Build [1-{}] (max {}), 'all', or 'skip': ",
        "→".magenta(), features.len(), max);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input == "skip" || input == "s" || input.is_empty() {
        return Ok(vec![]);
    }

    if input == "all" || input == "a" {
        return Ok(features[..max].to_vec());
    }

    let mut selected = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if let Ok(n) = part.parse::<usize>() {
            if n >= 1 && n <= features.len() {
                selected.push(features[n - 1].clone());
            }
        }
    }
    selected.dedup_by_key(|f| f.title.clone());
    selected.truncate(max);
    Ok(selected)
}

// ─── Approval gate: TUI ───────────────────────────────────────────────────

async fn tui_approval_gate(
    tui_state: &Option<SharedTuiState>,
    features: &[DreamFeature],
    parallel: u32,
) -> Result<Vec<DreamFeature>> {
    // Write pending approval to TUI state
    if let Some(ref s) = tui_state {
        if let Ok(mut st) = s.lock() {
            st.pending_approval = Some(ApprovalRequest {
                features: features.iter().map(|f| crate::tui::DreamFeatureView {
                    title: f.title.clone(),
                    description: f.description.chars().take(120).collect(),
                    wow_factor: f.wow_factor,
                }).collect(),
                max_select: parallel as usize,
            });
        }
    }

    // Poll until TUI user makes a selection (approved_indices set to Some)
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        if let Some(ref s) = tui_state {
            if let Ok(mut st) = s.lock() {
                if let Some(indices) = st.approved_indices.take() {
                    st.pending_approval = None;
                    let approved: Vec<DreamFeature> = indices.into_iter()
                        .filter(|&i| i < features.len())
                        .map(|i| features[i].clone())
                        .collect();
                    return Ok(approved);
                }
            }
        }
    }
}

// ─── Dream types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DreamFeature {
    pub title: String,
    pub description: String,
    pub wow_factor: u8,
    pub files: Vec<String>,
    pub creative_notes: String,
}

// ─── Prompts ──────────────────────────────────────────────────────────────

fn build_dreamer_prompt(proj: &Project, prior_dreams: &str, iteration: u32) -> String {
    format!(
        r#"# Subspace Dream Mode — Dreamer Agent (Opus)

You are the **Dreamer** — an AI whose sole job is to imagine remarkable new features
for this project. You don't fix bugs. You don't close TODOs. You INVENT.

## Project Stack: {stack}
## Dream Cycle: {iteration}

## Prior Dreams (already built — don't repeat these)

{prior_dreams}

## Your Mission

Read the project thoroughly:
- What does it do today?
- What could make it **10x more powerful**?
- What would make a developer say "I didn't know I needed this"?
- What crossover features from other domains would be surprising and useful?
- What would the ideal version of this project look like in 3 years?

## Output Format

Generate 3-6 dream features, ranked by wow-factor (10 = mind-blowing):

<dream_features>

### Feature 1: [Compelling Name]
**Wow Factor:** 9/10
**The Dream:** [2-3 sentences describing the vision, not the implementation]
**Why It's Remarkable:** [What makes this special]
**How To Build It:** [Concrete implementation steps for a Sonnet agent]
**Files To Create/Modify:**
- new/or/existing/file.rs
**Creative Notes:** [Unusual approaches, inspirations, wild ideas]

### Feature 2: ...

</dream_features>

## Rules

- Be BOLD. Don't suggest "add error handling". Suggest "time-travel debugging".
- Each feature must be BUILDABLE by a Sonnet agent in one session
- Features must be PARALLELIZABLE (different files)
- Lean into the project's strengths — amplify what's already interesting"#,
        stack = proj.stack,
        iteration = iteration,
        prior_dreams = if prior_dreams.is_empty() {
            "None yet — this is the first dream cycle!".to_string()
        } else {
            prior_dreams.to_string()
        },
    )
}

fn build_builder_prompt(proj: &Project, feature: &DreamFeature) -> String {
    format!(
        r#"# Subspace Dream Mode — Builder Agent (Sonnet)

You are a **Builder** in dream mode. The Dreamer has envisioned a new feature
and the user has **approved it for building**. Implement it fully.

## Stack: {stack}

## Feature: {title}
**Wow Factor:** {wow}/10

## The Dream
{description}

## How To Build It
{notes}

## Files to Create/Modify
{files}

## Rules
- Implement the FULL feature, not a stub
- Write tests if the project has a test suite
- Update docs/README to mention the new feature
- Commit with: `dream: {title}`
- Append to DREAMS.md: `## ✨ {title}\n[1-2 sentence summary]`

Make this feature something a user will discover and go "whoa"."#,
        stack = proj.stack,
        title = feature.title,
        wow = feature.wow_factor,
        description = feature.description,
        notes = feature.creative_notes,
        files = feature.files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n"),
    )
}

fn build_dream_merge_prompt(
    proj: &Project, base: &str, agents: &[String], subspace_dir: &Path,
) -> Result<String> {
    let mut details = String::new();
    for id in agents {
        let diff = worktree::diff_from_base(&proj.dir, id, base).unwrap_or_default();
        let log = worktree::branch_log(&proj.dir, id, base).unwrap_or_default();
        details.push_str(&format!(
            "### subspace/{}\n```\n{}\n```\n```diff\n{}\n```\n---\n\n",
            id, log.trim(), truncate(&diff, 4000),
        ));
    }

    Ok(format!(
        r#"# Dream Merge Agent (Haiku)

Merge these dream feature branches into `{}`.
Each branch adds a new user-approved creative feature — preserve ALL of them.

## Branches
{}

## Instructions
1. Merge each: `git merge subspace/<id> --no-ff -m "dream: merge <feature>"`
2. Resolve any conflicts (keep both features where possible)
3. Quick sanity check: `{}`
4. Do NOT remove any feature

{}
"#,
        base,
        agents.iter().map(|id| format!("- subspace/{}", id)).collect::<Vec<_>>().join("\n"),
        quality_checks(&proj.stack),
        details,
    ))
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn load_dreams(project_dir: &Path) -> String {
    std::fs::read_to_string(project_dir.join(DREAMS_FILE)).unwrap_or_default()
}

fn append_to_dreams(project_dir: &Path, features: &[DreamFeature], cycle: u32) -> Result<()> {
    let path = project_dir.join(DREAMS_FILE);
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let mut content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        "# DREAMS.md\n\n*Features invented by Subspace Dream Mode.*\n\n".to_string()
    };
    content.push_str(&format!("\n## Cycle {} — {}\n\n", cycle, now));
    for f in features {
        content.push_str(&format!(
            "### ✨ {} (wow: {}/10)\n{}\n\n",
            f.title, f.wow_factor, f.description
        ));
    }
    std::fs::write(&path, content)?;
    Ok(())
}

fn parse_dream_features(output: &str) -> Vec<DreamFeature> {
    let mut features = Vec::new();
    let mut in_features = false;
    let mut current: Option<DreamFeature> = None;
    let mut in_section = "";

    for line in output.lines() {
        let t = line.trim();
        if t == "<dream_features>" { in_features = true; continue; }
        if t == "</dream_features>" {
            if let Some(f) = current.take() { features.push(f); }
            break;
        }
        if !in_features { continue; }

        if t.starts_with("### Feature") {
            if let Some(f) = current.take() { features.push(f); }
            let title = t.trim_start_matches("### Feature")
                .trim_start_matches(|c: char| c.is_numeric() || c == ':' || c == ' ')
                .trim().to_string();
            current = Some(DreamFeature {
                title, description: String::new(), wow_factor: 7,
                files: Vec::new(), creative_notes: String::new(),
            });
            in_section = "";
        } else if let Some(ref mut f) = current {
            if t.starts_with("**Wow Factor:**") {
                f.wow_factor = t.chars().filter(|c| c.is_ascii_digit()).next()
                    .and_then(|c| c.to_digit(10)).unwrap_or(7) as u8;
            } else if t.starts_with("**The Dream:**") {
                in_section = "dream";
                f.description.push_str(t.trim_start_matches("**The Dream:**").trim());
            } else if t.starts_with("**How To Build It:**") || t.starts_with("**Creative Notes:**") {
                in_section = "notes";
                f.creative_notes.push_str(
                    t.trim_start_matches("**How To Build It:**")
                     .trim_start_matches("**Creative Notes:**").trim()
                );
            } else if t.starts_with("**Files To") { in_section = "files"; }
            else if t.starts_with("**") { in_section = ""; }
            else {
                match in_section {
                    "dream" => { if !t.is_empty() { f.description.push(' '); f.description.push_str(t); } }
                    "notes" => { if !t.is_empty() { f.creative_notes.push(' '); f.creative_notes.push_str(t); } }
                    "files" => {
                        if t.starts_with("- ") {
                            let file = t[2..].trim().to_string();
                            if !file.is_empty() { f.files.push(file); }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    features.sort_by(|a, b| b.wow_factor.cmp(&a.wow_factor));
    features
}

fn quality_checks(stack: &str) -> String {
    let mut c = Vec::new();
    if stack.contains("rust") { c.push("cargo check"); }
    if stack.contains("node") || stack.contains("typescript") { c.push("npm run build"); }
    if stack.contains("python") { c.push("python -m py_compile"); }
    if stack.contains("go") { c.push("go build ./..."); }
    if c.is_empty() { c.push("echo ok"); }
    c.join(" && ")
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
