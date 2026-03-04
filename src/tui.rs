//! TUI dashboard for Subspace — live view of the swarm.
//!
//! Layout:
//! ┌─────────────────────────────────────────┐
//! │  SUBSPACE  project-name  35%  iter 2/10 │  ← header
//! ├──────────────┬──────────────────────────┤
//! │  AGENTS      │  LOG                     │
//! │  🧠 planner  │  [14:32] planner: read.. │
//! │    ✓ done    │  [14:33] exec-1: cargo.. │
//! │  🔨 exec-1   │  [14:33] exec-2: added.. │
//! │    ● running │  [14:34] merger: merging  │
//! │  🔨 exec-2   │                          │
//! │    ● running │                          │
//! │  🔀 merger   │                          │
//! │    ◌ waiting │                          │
//! ├──────────────┴──────────────────────────┤
//! │  TASKS  [ ] 🔴 Fix auth  [~] 🟠 Tests  │  ← task strip
//! └─────────────────────────────────────────┘

use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

/// Shared state updated by the swarm, read by the TUI
#[derive(Debug, Clone, Default)]
pub struct TuiState {
    pub project_name: String,
    pub completion_pct: u8,
    pub iteration: u32,
    pub max_iter: u32,
    pub agents: Vec<AgentView>,
    pub log: Vec<LogEntry>,
    pub tasks_pending: u32,
    pub tasks_done: u32,
    pub phase: Phase,
    pub done: bool,
    /// Pending approval request — dreamer wants user to pick features
    pub pending_approval: Option<ApprovalRequest>,
    /// Set by TUI when user approves — indices into pending_approval.features
    pub approved_indices: Option<Vec<usize>>,
}

/// A dream feature shown in the approval picker
#[derive(Debug, Clone)]
pub struct DreamFeatureView {
    pub title: String,
    pub description: String,
    pub wow_factor: u8,
}

/// Approval request sent from dream mode to TUI
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub features: Vec<DreamFeatureView>,
    pub max_select: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Phase {
    #[default]
    Starting,
    Planning,
    Executing,
    Merging,
    Complete,
}

#[derive(Debug, Clone)]
pub struct AgentView {
    pub id: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub current_tool: Option<String>,
    pub files_touched: u32,
    pub tool_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentRole { Planner, Executor, Merger }

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus { Waiting, Running, Done, Failed }

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub time: String,
    pub agent: String,
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone)]
pub enum LogLevel { Info, Success, Warning, Error }

impl TuiState {
    pub fn log(&mut self, agent: &str, msg: &str, level: LogLevel) {
        self.log.push(LogEntry {
            time: Local::now().format("%H:%M:%S").to_string(),
            agent: agent.to_string(),
            message: msg.to_string(),
            level,
        });
        // Cap log at 1000 entries
        if self.log.len() > 1000 {
            self.log.drain(0..100);
        }
    }
}

pub type SharedTuiState = Arc<Mutex<TuiState>>;

pub fn new_state() -> SharedTuiState {
    Arc::new(Mutex::new(TuiState::default()))
}

#[macro_export]
macro_rules! tui_log {
    ($state:expr, $agent:expr, $level:expr, $($arg:tt)*) => {
        if let Some(ref s) = $state {
            if let Ok(mut st) = s.lock() {
                st.log($agent, &format!($($arg)*), $level);
            }
        }
    };
}

#[macro_export]
macro_rules! tui_phase {
    ($state:expr, $phase:expr) => {
        if let Some(ref s) = $state {
            if let Ok(mut st) = s.lock() {
                st.phase = $phase;
            }
        }
    };
}

#[macro_export]
macro_rules! tui_agent {
    ($state:expr, $view:expr) => {
        if let Some(ref s) = $state {
            if let Ok(mut st) = s.lock() {
                let id = $view.id.clone();
                if let Some(existing) = st.agents.iter_mut().find(|a| a.id == id) {
                    *existing = $view;
                } else {
                    st.agents.push($view);
                }
            }
        }
    };
}

/// Run the TUI in a blocking loop until state.done is true
pub fn run(state: SharedTuiState) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Approval picker state (local to TUI thread)
    let mut picker_selected: Vec<bool> = Vec::new();
    let mut picker_cursor: usize = 0;

    loop {
        {
            let st = state.lock().unwrap();

            // Sync picker_selected length with pending_approval features
            if let Some(ref req) = st.pending_approval {
                if picker_selected.len() != req.features.len() {
                    picker_selected = vec![false; req.features.len()];
                    picker_cursor = 0;
                }
                terminal.draw(|f| render_approval(f, req, &picker_selected, picker_cursor))?;
            } else {
                picker_selected.clear();
                terminal.draw(|f| render(f, &st))?;
            }

            if st.done { break; }
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let has_approval = state.lock().unwrap().pending_approval.is_some();

                    if has_approval {
                        let req_len = state.lock().unwrap()
                            .pending_approval.as_ref().map(|r| r.features.len()).unwrap_or(0);
                        let max_sel = state.lock().unwrap()
                            .pending_approval.as_ref().map(|r| r.max_select).unwrap_or(1);

                        match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                picker_cursor = picker_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                picker_cursor = (picker_cursor + 1).min(req_len.saturating_sub(1));
                            }
                            KeyCode::Char(' ') => {
                                // Toggle selection (respect max_select)
                                if picker_cursor < picker_selected.len() {
                                    let currently = picker_selected[picker_cursor];
                                    let count = picker_selected.iter().filter(|&&x| x).count();
                                    if currently {
                                        picker_selected[picker_cursor] = false;
                                    } else if count < max_sel {
                                        picker_selected[picker_cursor] = true;
                                    }
                                }
                            }
                            KeyCode::Char('a') => {
                                // Select all up to max
                                let mut n = 0;
                                for s in picker_selected.iter_mut() {
                                    if n < max_sel { *s = true; n += 1; }
                                    else { *s = false; }
                                }
                            }
                            KeyCode::Enter => {
                                // Submit selection
                                let indices: Vec<usize> = picker_selected.iter()
                                    .enumerate()
                                    .filter(|(_, &s)| s)
                                    .map(|(i, _)| i)
                                    .collect();
                                if let Ok(mut st) = state.lock() {
                                    st.approved_indices = Some(indices);
                                }
                                picker_selected.clear();
                            }
                            KeyCode::Esc | KeyCode::Char('s') => {
                                // Skip (submit empty)
                                if let Ok(mut st) = state.lock() {
                                    st.approved_indices = Some(vec![]);
                                }
                                picker_selected.clear();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn render(f: &mut Frame, state: &TuiState) {
    let size = f.area();

    // Outer layout: header / body / footer
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(0),     // body
            Constraint::Length(3),  // task strip
        ])
        .split(size);

    render_header(f, outer[0], state);
    render_body(f, outer[1], state);
    render_task_strip(f, outer[2], state);
}

fn render_header(f: &mut Frame, area: Rect, state: &TuiState) {
    let phase_str = match state.phase {
        Phase::Starting => "starting",
        Phase::Planning => "🧠 planning",
        Phase::Executing => "🔨 executing",
        Phase::Merging => "🔀 merging",
        Phase::Complete => "✅ complete",
    };

    let title = format!(
        " SUBSPACE  {}  {}%  iter {}/{}  [{}] ",
        if state.project_name.is_empty() { "detecting..." } else { &state.project_name },
        state.completion_pct,
        state.iteration,
        state.max_iter,
        phase_str,
    );

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default());
    f.render_widget(header, area);
}

fn render_body(f: &mut Frame, area: Rect, state: &TuiState) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(area);

    render_agents(f, body[0], state);
    render_log(f, body[1], state);
}

fn render_agents(f: &mut Frame, area: Rect, state: &TuiState) {
    let mut items: Vec<ListItem> = Vec::new();

    if state.agents.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  waiting for agents...",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    for agent in &state.agents {
        let role_icon = match agent.role {
            AgentRole::Planner => "🧠",
            AgentRole::Executor => "🔨",
            AgentRole::Merger => "🔀",
        };

        let (status_icon, status_color) = match agent.status {
            AgentStatus::Waiting => ("◌", Color::DarkGray),
            AgentStatus::Running => ("●", Color::Green),
            AgentStatus::Done => ("✓", Color::Blue),
            AgentStatus::Failed => ("✗", Color::Red),
        };

        // Agent name line
        items.push(ListItem::new(Line::from(vec![
            Span::raw(format!("  {} ", role_icon)),
            Span::styled(agent.id.clone(), Style::default().add_modifier(Modifier::BOLD)),
        ])));

        // Status line
        items.push(ListItem::new(Line::from(vec![
            Span::raw("    "),
            Span::styled(status_icon, Style::default().fg(status_color)),
            Span::styled(
                format!(" {} tools, {} files",
                    agent.tool_count, agent.files_touched),
                Style::default().fg(Color::DarkGray),
            ),
        ])));

        // Current tool (if running)
        if let Some(tool) = &agent.current_tool {
            let truncated = if tool.len() > 22 { &tool[..22] } else { tool };
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    format!("↳ {}", truncated),
                    Style::default().fg(Color::Yellow),
                ),
            ])));
        }

        items.push(ListItem::new(Line::from("")));
    }

    let list = List::new(items)
        .block(Block::default().title(" Agents ").borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(list, area);
}

fn render_log(f: &mut Frame, area: Rect, state: &TuiState) {
    let height = area.height.saturating_sub(2) as usize;
    let start = state.log.len().saturating_sub(height);
    let visible = &state.log[start..];

    let items: Vec<ListItem> = visible.iter().map(|entry| {
        let time_span = Span::styled(
            format!("[{}] ", entry.time),
            Style::default().fg(Color::DarkGray),
        );
        let agent_span = Span::styled(
            format!("{}: ", entry.agent),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        );
        let msg_color = match entry.level {
            LogLevel::Success => Color::Green,
            LogLevel::Warning => Color::Yellow,
            LogLevel::Error => Color::Red,
            LogLevel::Info => Color::White,
        };
        let msg_span = Span::styled(
            entry.message.clone(),
            Style::default().fg(msg_color),
        );
        ListItem::new(Line::from(vec![time_span, agent_span, msg_span]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" Log  (q to quit) ").borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(list, area);
}

fn render_task_strip(f: &mut Frame, area: Rect, state: &TuiState) {
    let text = format!(
        "  Tasks: {} pending  {}  done  │  Phase: {}",
        state.tasks_pending,
        state.tasks_done,
        match state.phase {
            Phase::Starting => "Starting up...",
            Phase::Planning => "Planner analyzing codebase",
            Phase::Executing => "Executor agents running in parallel worktrees",
            Phase::Merging => "Haiku merger combining branches",
            Phase::Complete => "All done! 🎉",
        }
    );

    let para = Paragraph::new(text)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .block(Block::default());
    f.render_widget(para, area);
}

/// Approval picker — full-screen overlay when dreamer has results waiting
fn render_approval(
    f: &mut Frame,
    req: &ApprovalRequest,
    selected: &[bool],
    cursor: usize,
) {
    let size = f.area();

    let block = Block::default()
        .title(" ✨ DREAMER RESULTS — Pick features to build (Space=toggle, Enter=confirm, s=skip, a=all) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(size);
    f.render_widget(block, size);

    let sel_count = selected.iter().filter(|&&x| x).count();
    let help = Paragraph::new(format!(
        "  Selected: {}/{}  (max {})    ↑↓/jk to move, Space to toggle, Enter to build, s to skip",
        sel_count, selected.len(), req.max_select
    )).style(Style::default().fg(Color::DarkGray));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    f.render_widget(help, layout[0]);

    let items: Vec<ListItem> = req.features.iter().enumerate().map(|(i, feat)| {
        let is_cursor = i == cursor;
        let is_sel = selected.get(i).copied().unwrap_or(false);

        let check = if is_sel { "◉" } else { "○" };
        let wow_bar = "█".repeat(feat.wow_factor as usize / 2);
        let wow_color = if feat.wow_factor >= 8 { Color::Green }
            else if feat.wow_factor >= 6 { Color::Yellow }
            else { Color::DarkGray };

        let cursor_mark = if is_cursor { "▶ " } else { "  " };

        let mut spans = vec![
            Span::styled(cursor_mark, Style::default().fg(Color::Magenta)),
            Span::styled(
                format!("{} ", check),
                Style::default().fg(if is_sel { Color::Green } else { Color::DarkGray }),
            ),
            Span::styled(
                format!("{:>2}/10 ", feat.wow_factor),
                Style::default().fg(wow_color),
            ),
            Span::styled(wow_bar, Style::default().fg(wow_color)),
            Span::raw("  "),
            Span::styled(
                feat.title.clone(),
                Style::default()
                    .fg(if is_cursor { Color::White } else { Color::Gray })
                    .add_modifier(if is_cursor { Modifier::BOLD } else { Modifier::empty() }),
            ),
        ];

        let mut lines = vec![Line::from(spans)];
        // Description line
        lines.push(Line::from(vec![
            Span::raw("       "),
            Span::styled(
                feat.description.chars().take(90).collect::<String>(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));

        ListItem::new(lines)
    }).collect();

    let list = List::new(items);
    f.render_widget(list, layout[1]);
}
