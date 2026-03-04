use anyhow::{bail, Context, Result};
use colored::*;
use std::path::{Path, PathBuf};

use crate::config;

pub struct Project {
    pub dir: PathBuf,
    pub stack: String,
    pub goals: String,
    pub progress: String,
}

pub fn resolve_dir(dir: Option<String>) -> Result<PathBuf> {
    match dir {
        Some(d) => {
            let p = PathBuf::from(&d);
            if !p.exists() {
                bail!("Directory does not exist: {}", d);
            }
            Ok(std::fs::canonicalize(p)?)
        }
        None => Ok(std::env::current_dir()?),
    }
}

pub fn init(dir: &Path) -> Result<()> {
    let subspace_dir = dir.join(".subspace");
    std::fs::create_dir_all(&subspace_dir)?;

    let goals_path = dir.join("goals.md");
    if !goals_path.exists() {
        std::fs::write(
            &goals_path,
            r#"# Project Goals

## What We Are Building
[Describe your project here]

## Specific Goals
- [ ] Goal 1
- [ ] Goal 2
- [ ] Goal 3

## Constraints
- [Any constraints or patterns to follow]

## Quality Requirements
- All tests must pass
- No compiler warnings
"#,
        )?;
        println!("{} Created goals.md", "✓".green());
    } else {
        println!("{} goals.md already exists", "·".yellow());
    }

    let progress_path = dir.join("progress.md");
    if !progress_path.exists() {
        std::fs::write(
            &progress_path,
            format!(
                "# Subspace Progress Log\nStarted: {}\n---\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ),
        )?;
        println!("{} Created progress.md", "✓".green());
    }

    println!("{} Created .subspace/", "✓".green());
    println!();
    println!("Edit {} to define your goals, then run:", "goals.md".cyan());
    println!("  {} run", "subspace".bold());
    Ok(())
}

pub fn load(dir: &Path) -> Result<Project> {
    let goals_path = dir.join("goals.md");
    if !goals_path.exists() {
        bail!(
            "No goals.md found in {}. Run `subspace init` first.",
            dir.display()
        );
    }

    let goals = std::fs::read_to_string(&goals_path)
        .context("Failed to read goals.md")?;

    let progress_path = dir.join("progress.md");
    let progress = if progress_path.exists() {
        std::fs::read_to_string(&progress_path)?
    } else {
        "No progress yet.".to_string()
    };

    let stack = detect_stack(dir);

    Ok(Project {
        dir: dir.to_path_buf(),
        stack,
        goals,
        progress,
    })
}

/// Load project or auto-init (goals come from CLI now, not goals.md)
pub fn load_or_init(dir: &Path) -> Result<Project> {
    let subspace_dir = dir.join(".subspace");
    if !subspace_dir.exists() {
        std::fs::create_dir_all(&subspace_dir)?;
    }

    let progress_path = dir.join("progress.md");
    let progress = if progress_path.exists() {
        std::fs::read_to_string(&progress_path)?
    } else {
        "No progress yet.".to_string()
    };

    let goals_path = dir.join("goals.md");
    let goals = if goals_path.exists() {
        std::fs::read_to_string(&goals_path)?
    } else {
        String::new()
    };

    Ok(Project {
        dir: dir.to_path_buf(),
        stack: detect_stack(dir),
        goals,
        progress,
    })
}

pub fn detect_stack(dir: &Path) -> String {
    let mut stack = Vec::new();
    let checks = [
        ("package.json", "node"),
        ("tsconfig.json", "typescript"),
        ("Cargo.toml", "rust"),
        ("go.mod", "go"),
        ("requirements.txt", "python"),
        ("pyproject.toml", "python"),
        ("setup.py", "python"),
        ("Gemfile", "ruby"),
        ("pom.xml", "java-maven"),
        ("build.gradle", "java-gradle"),
        ("build.gradle.kts", "kotlin-gradle"),
        ("composer.json", "php"),
        ("mix.exs", "elixir"),
        ("pubspec.yaml", "dart-flutter"),
        ("Package.swift", "swift"),
        ("Makefile", "make"),
        ("CMakeLists.txt", "cmake"),
        ("Dockerfile", "docker"),
        ("docker-compose.yml", "docker-compose"),
        ("next.config.js", "nextjs"),
        ("next.config.ts", "nextjs"),
        ("next.config.mjs", "nextjs"),
        ("vite.config.ts", "vite"),
        ("vite.config.js", "vite"),
        ("svelte.config.js", "svelte"),
        ("angular.json", "angular"),
        ("tailwind.config.js", "tailwind"),
        ("tailwind.config.ts", "tailwind"),
    ];

    for (file, name) in checks {
        if dir.join(file).exists() && !stack.contains(&name) {
            stack.push(name);
        }
    }

    if stack.is_empty() {
        "unknown".to_string()
    } else {
        stack.join(",")
    }
}

pub fn show_status(proj: &Project) -> Result<()> {
    println!("{}", "═══ Subspace Status ═══".bold());
    println!("Project:  {}", proj.dir.display().to_string().cyan());
    println!("Stack:    {}", proj.stack.yellow());
    println!();

    if let Some(state) = config::load_state(&proj.dir)? {
        println!("Session:  {}", state.session_name.green());
        println!("Iteration: {}", state.iteration);
        println!();

        let running = state.agents.iter().filter(|a| a.status == config::AgentStatus::Running).count();
        let completed = state.agents.iter().filter(|a| a.status == config::AgentStatus::Completed).count();
        let failed = state.agents.iter().filter(|a| a.status == config::AgentStatus::Failed).count();

        println!("Agents: {} running, {} completed, {} failed",
            running.to_string().green(),
            completed.to_string().blue(),
            failed.to_string().red(),
        );

        if !state.conflicts.is_empty() {
            println!();
            println!("{}", "⚠ File Conflicts Detected!".red().bold());
            for c in &state.conflicts {
                println!("  {} — modified by: {}", c.file.yellow(), c.agents.join(", "));
            }
        }
    } else {
        println!("{}", "No active swarm. Run `subspace run` to start.".dimmed());
    }

    Ok(())
}

pub fn session_name(dir: &Path) -> String {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());
    format!("subspace-{}", name)
}

pub fn append_progress(dir: &Path, entry: &str) -> Result<()> {
    use std::io::Write;
    let path = dir.join("progress.md");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "\n{}", entry)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "rust");
    }

    #[test]
    fn test_detect_node() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "node");
    }

    #[test]
    fn test_detect_python_requirements() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "python");
    }

    #[test]
    fn test_detect_python_pyproject() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "python");
    }

    #[test]
    fn test_detect_multiple() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        std::fs::write(dir.path().join("Dockerfile"), "").unwrap();
        let result = detect_stack(dir.path());
        assert!(result.contains("rust"), "expected 'rust' in '{}'", result);
        assert!(result.contains("docker"), "expected 'docker' in '{}'", result);
    }

    #[test]
    fn test_detect_unknown() {
        let dir = tempdir().unwrap();
        assert_eq!(detect_stack(dir.path()), "unknown");
    }

    #[test]
    fn test_detect_nextjs() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("next.config.js"), "").unwrap();
        std::fs::write(dir.path().join("package.json"), "").unwrap();
        let result = detect_stack(dir.path());
        assert!(result.contains("nextjs"), "expected 'nextjs' in '{}'", result);
        assert!(result.contains("node"), "expected 'node' in '{}'", result);
    }

    #[test]
    fn test_session_name() {
        let dir = PathBuf::from("/tmp/my-cool-project");
        assert_eq!(session_name(&dir), "subspace-my-cool-project");
    }
}
