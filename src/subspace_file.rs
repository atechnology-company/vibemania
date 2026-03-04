use anyhow::Result;
use std::path::Path;

/// The subspace.md file — Subspace's persistent brain for a project.
/// Written and updated by the planner each iteration.
#[derive(Debug, Clone, Default)]
pub struct SubspaceFile {
    pub project_name: String,
    pub description: String,
    pub completion_pct: u8,
    pub roadmap: Vec<RoadmapItem>,
    pub tasks: Vec<Task>,
    pub completed: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct RoadmapItem {
    pub phase: String,
    pub description: String,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub files: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskPriority {
    Critical,   // 🔴 broken things
    High,       // 🟠 missing features
    Medium,     // 🟡 tests/error handling
    Low,        // 🟢 docs/polish
    Nice,       // 🔵 optimization
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Skipped,
}

impl SubspaceFile {
    pub fn path(project_dir: &Path) -> std::path::PathBuf {
        project_dir.join("subspace.md")
    }

    /// Load from subspace.md if it exists
    pub fn load(project_dir: &Path) -> Result<Option<Self>> {
        let path = Self::path(project_dir);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(Self::parse(&content)))
    }

    /// Load or return empty default
    pub fn load_or_default(project_dir: &Path) -> Result<Self> {
        Ok(Self::load(project_dir)?.unwrap_or_default())
    }

    /// Parse subspace.md content
    pub fn parse(content: &str) -> Self {
        let mut sf = SubspaceFile::default();
        let mut in_section = "";
        let mut task_buf: Option<Task> = None;
        let mut task_id = 0u32;

        for line in content.lines() {
            let trimmed = line.trim();

            // Section headers
            if trimmed.starts_with("## ") {
                // Flush pending task
                if let Some(t) = task_buf.take() {
                    sf.tasks.push(t);
                }
                in_section = if trimmed.contains("Project") { "project" }
                    else if trimmed.contains("Roadmap") { "roadmap" }
                    else if trimmed.contains("Tasks") || trimmed.contains("Todo") || trimmed.contains("Backlog") { "tasks" }
                    else if trimmed.contains("Completed") || trimmed.contains("Done") { "completed" }
                    else if trimmed.contains("Notes") { "notes" }
                    else { "" };
                continue;
            }

            match in_section {
                "project" => {
                    if trimmed.starts_with("- **Name") || trimmed.starts_with("**Name") {
                        sf.project_name = extract_value(trimmed);
                    } else if trimmed.starts_with("- **Completion") || trimmed.starts_with("**Completion") {
                        let val = extract_value(trimmed).replace('%', "");
                        sf.completion_pct = val.trim().parse().unwrap_or(0);
                    } else if trimmed.starts_with("- **Description") || trimmed.starts_with("**Description") {
                        sf.description = extract_value(trimmed);
                    }
                }
                "roadmap" => {
                    if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
                        sf.roadmap.push(RoadmapItem {
                            phase: String::new(),
                            description: trimmed[6..].trim().to_string(),
                            done: true,
                        });
                    } else if trimmed.starts_with("- [ ]") {
                        sf.roadmap.push(RoadmapItem {
                            phase: String::new(),
                            description: trimmed[6..].trim().to_string(),
                            done: false,
                        });
                    } else if let Some(stripped) = trimmed.strip_prefix("- ") {
                        sf.roadmap.push(RoadmapItem {
                            phase: String::new(),
                            description: stripped.trim().to_string(),
                            done: false,
                        });
                    }
                }
                "tasks" => {
                    if trimmed.starts_with("### ") {
                        if let Some(t) = task_buf.take() {
                            sf.tasks.push(t);
                        }
                        task_id += 1;
                        let title_raw = trimmed.trim_start_matches("### ").trim();
                        let (status, title) = parse_task_header(title_raw);
                        task_buf = Some(Task {
                            id: task_id,
                            title,
                            priority: TaskPriority::Medium,
                            status,
                            files: Vec::new(),
                            description: String::new(),
                        });
                    } else if let Some(ref mut t) = task_buf {
                        if let Some(stripped) = trimmed.strip_prefix("- ") {
                            if stripped.contains('/') || stripped.contains('.') {
                                t.files.push(stripped.trim().to_string());
                            }
                        } else if trimmed.contains("🔴") || trimmed.to_lowercase().contains("critical") {
                            t.priority = TaskPriority::Critical;
                        } else if trimmed.contains("🟠") || trimmed.to_lowercase().contains("high") {
                            t.priority = TaskPriority::High;
                        } else if trimmed.contains("🟡") || trimmed.to_lowercase().contains("medium") {
                            t.priority = TaskPriority::Medium;
                        } else if trimmed.contains("🟢") || trimmed.to_lowercase().contains("low") {
                            t.priority = TaskPriority::Low;
                        } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                            if !t.description.is_empty() { t.description.push('\n'); }
                            t.description.push_str(trimmed);
                        }
                    }
                }
                "completed" => {
                    if let Some(stripped) = trimmed.strip_prefix("- ") {
                        sf.completed.push(stripped.trim().to_string());
                    }
                }
                "notes" => {
                    if !trimmed.is_empty() {
                        if !sf.notes.is_empty() { sf.notes.push('\n'); }
                        sf.notes.push_str(trimmed);
                    }
                }
                _ => {}
            }
        }

        // Flush last task
        if let Some(t) = task_buf {
            sf.tasks.push(t);
        }

        sf
    }

    /// Get pending tasks (todo + in-progress), sorted by priority
    pub fn pending_tasks(&self) -> Vec<&Task> {
        let mut tasks: Vec<&Task> = self.tasks.iter()
            .filter(|t| t.status == TaskStatus::Todo || t.status == TaskStatus::InProgress)
            .collect();
        tasks.sort_by_key(|t| match t.priority {
            TaskPriority::Critical => 0,
            TaskPriority::High => 1,
            TaskPriority::Medium => 2,
            TaskPriority::Low => 3,
            TaskPriority::Nice => 4,
        });
        tasks
    }

    /// Check if we have explicit tasks defined
    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }

    /// Serialize back to markdown
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Subspace\n\n");
        out.push_str("> Autonomous project completion file. Managed by Subspace AI swarm.\n\n");

        out.push_str("## Project\n\n");
        out.push_str(&format!("- **Name:** {}\n", self.project_name));
        out.push_str(&format!("- **Description:** {}\n", self.description));
        out.push_str(&format!("- **Completion:** {}%\n\n", self.completion_pct));

        if !self.roadmap.is_empty() {
            out.push_str("## Roadmap\n\n");
            for item in &self.roadmap {
                let check = if item.done { "[x]" } else { "[ ]" };
                out.push_str(&format!("- {} {}\n", check, item.description));
            }
            out.push('\n');
        }

        if !self.tasks.is_empty() {
            out.push_str("## Tasks\n\n");
            for task in &self.tasks {
                let status = match task.status {
                    TaskStatus::Todo => "[ ]",
                    TaskStatus::InProgress => "[~]",
                    TaskStatus::Done => "[x]",
                    TaskStatus::Skipped => "[-]",
                };
                let priority = match task.priority {
                    TaskPriority::Critical => "🔴",
                    TaskPriority::High => "🟠",
                    TaskPriority::Medium => "🟡",
                    TaskPriority::Low => "🟢",
                    TaskPriority::Nice => "🔵",
                };
                out.push_str(&format!("### {} {} {}\n", status, priority, task.title));
                if !task.description.is_empty() {
                    out.push_str(&task.description);
                    out.push('\n');
                }
                if !task.files.is_empty() {
                    for f in &task.files {
                        out.push_str(&format!("- {}\n", f));
                    }
                }
                out.push('\n');
            }
        }

        if !self.completed.is_empty() {
            out.push_str("## Completed\n\n");
            for item in &self.completed {
                out.push_str(&format!("- {}\n", item));
            }
            out.push('\n');
        }

        if !self.notes.is_empty() {
            out.push_str("## Notes\n\n");
            out.push_str(&self.notes);
            out.push('\n');
        }

        out
    }

    pub fn save(&self, project_dir: &Path) -> Result<()> {
        std::fs::write(Self::path(project_dir), self.to_markdown())?;
        Ok(())
    }
}

fn extract_value(line: &str) -> String {
    if let Some(pos) = line.find(":**") {
        line[pos + 3..].trim().trim_start_matches('*').trim().to_string()
    } else if let Some(pos) = line.find(": ") {
        line[pos + 2..].trim().to_string()
    } else {
        line.to_string()
    }
}

fn parse_task_header(raw: &str) -> (TaskStatus, String) {
    if let Some(stripped) = raw.strip_prefix("[x]").or_else(|| raw.strip_prefix("[X]")) {
        (TaskStatus::Done, stripped.trim().to_string())
    } else if let Some(stripped) = raw.strip_prefix("[~]") {
        (TaskStatus::InProgress, stripped.trim().to_string())
    } else if let Some(stripped) = raw.strip_prefix("[-]") {
        (TaskStatus::Skipped, stripped.trim().to_string())
    } else if let Some(stripped) = raw.strip_prefix("[ ]") {
        (TaskStatus::Todo, stripped.trim().to_string())
    } else {
        (TaskStatus::Todo, raw.to_string())
    }
}
