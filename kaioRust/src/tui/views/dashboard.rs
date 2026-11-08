use crate::tui::app::{DaemonConnectionState, TuiApp};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render_dashboard(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    // 1. Workspace Health Widget
    let health_color = if app.health_score >= 80 {
        Color::Green
    } else if app.health_score >= 60 {
        Color::Yellow
    } else {
        Color::Red
    };

    let daemon_str = match app.daemon_state {
        DaemonConnectionState::Connected => ("Connected 🟢", Color::Green),
        DaemonConnectionState::Disconnected => ("Offline 🔴", Color::Red),
        DaemonConnectionState::Embedded => ("Embedded 📦", Color::Cyan),
    };

    let health_lines = vec![
        Line::from(vec![
            Span::raw("Repo:           "),
            Span::styled(
                &app.repo_name,
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ("),
            Span::styled(&app.branch, Style::default().fg(Color::Cyan)),
            Span::raw(")"),
        ]),
        Line::from(vec![
            Span::raw("Daemon:         "),
            Span::styled(daemon_str.0, Style::default().fg(daemon_str.1)),
        ]),
        Line::from(vec![
            Span::raw("Health Score:   "),
            Span::styled(
                format!("{}/100", app.health_score),
                Style::default()
                    .fg(health_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Doc Coverage:   "),
            Span::styled(
                format!("{}%", app.doc_coverage),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Total Files:    "),
            Span::styled(
                format!("{}", app.total_files),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::raw("Total Symbols:  "),
            Span::styled(
                format!("{}", app.total_symbols),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::raw("Active Locks:   "),
            Span::styled(
                if app.active_locks == 0 {
                    "None (Ready)"
                } else {
                    "Scan in Progress"
                },
                Style::default().fg(if app.active_locks == 0 {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ),
        ]),
    ];

    let health_block = Block::default()
        .borders(Borders::ALL)
        .title(" 📊 Workspace Health ");
    let health_widget = Paragraph::new(health_lines).block(health_block);
    frame.render_widget(health_widget, top_chunks[0]);

    // 2. Recent Commits & Memory Widget
    let commit_items: Vec<ListItem> = if app.recent_commits.is_empty() {
        vec![ListItem::new(
            "No recent commit memories recorded in database.",
        )]
    } else {
        app.recent_commits
            .iter()
            .map(|c| {
                let risk_color = match c.risk_level.to_lowercase().as_str() {
                    "high" | "critical" => Color::Red,
                    "medium" => Color::Yellow,
                    _ => Color::Green,
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("• {} ", c.commit_hash),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::raw(&c.message),
                    ]),
                    Line::from(vec![
                        Span::raw("   Files: "),
                        Span::styled(
                            format!("{}", c.files_changed),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(" | Symbols: "),
                        Span::styled(&c.symbols_delta, Style::default().fg(Color::LightBlue)),
                        Span::raw(" | Risk: "),
                        Span::styled(&c.risk_level, Style::default().fg(risk_color)),
                    ]),
                ])
            })
            .collect()
    };

    let commits_block = Block::default()
        .borders(Borders::ALL)
        .title(" 📜 Recent Commits & Memory ");
    let commits_list = List::new(commit_items).block(commits_block);
    frame.render_widget(commits_list, top_chunks[1]);

    // 3. Live Activity & Telemetry Feed
    let activity_items: Vec<ListItem> = app
        .live_activity
        .iter()
        .rev()
        .take(15)
        .rev()
        .map(|line| {
            let color = if line.contains("[WATCH]") || line.contains("[SYSTEM]") {
                Color::Cyan
            } else if line.contains("[CRITIC]") {
                Color::Green
            } else if line.contains("❌") || line.contains("ERROR") {
                Color::Red
            } else if line.contains("⚠️") {
                Color::Yellow
            } else {
                Color::White
            };
            ListItem::new(Span::styled(line, Style::default().fg(color)))
        })
        .collect();

    let activity_title = if app.is_paused {
        " ⚡ Live Activity / Agent Output [PAUSED] "
    } else {
        " ⚡ Live Activity / Agent Output "
    };

    let activity_block = Block::default().borders(Borders::ALL).title(activity_title);
    let activity_list = List::new(activity_items).block(activity_block);
    frame.render_widget(activity_list, chunks[1]);
}
