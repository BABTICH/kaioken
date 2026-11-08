use crate::tui::app::TuiApp;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub fn render_audit(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(area);

    // 1. Audit Log Table
    let header = Row::new(vec![
        Cell::from(Span::styled(
            "Time",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Actor",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Action",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Target",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Decision",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ])
    .style(Style::default().bg(Color::DarkGray).fg(Color::White))
    .bottom_margin(1);

    let rows: Vec<Row> = if app.audit_entries.is_empty() {
        vec![Row::new(vec![
            Cell::from("-"),
            Cell::from("-"),
            Cell::from("No audit events recorded"),
            Cell::from("-"),
            Cell::from("-"),
        ])]
    } else {
        app.audit_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let is_selected = idx == app.selected_audit_idx;
                let (decision_badge, decision_color) = if entry.approved {
                    ("🟢 ALLOWED", Color::Green)
                } else if entry.reason.as_deref().unwrap_or("").contains("redact") {
                    ("🟡 REDACTED", Color::Yellow)
                } else {
                    ("🔴 DENIED", Color::Red)
                };

                let time_display = if entry.created_at.len() > 19 {
                    &entry.created_at[11..19]
                } else {
                    &entry.created_at
                };

                let mut row = Row::new(vec![
                    Cell::from(Span::raw(time_display)),
                    Cell::from(Span::raw(&entry.actor)),
                    Cell::from(Span::styled(
                        &entry.action,
                        Style::default().fg(Color::Cyan),
                    )),
                    Cell::from(Span::raw(&entry.target)),
                    Cell::from(Span::styled(
                        decision_badge,
                        Style::default().fg(decision_color),
                    )),
                ]);

                if is_selected {
                    row = row.style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    );
                }

                row
            })
            .collect()
    };

    let title = format!(" 🛡️ Security & Audit Stream ({}) ", app.audit_entries.len());
    let table_block = Block::default().borders(Borders::ALL).title(title);

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Percentage(45),
            Constraint::Length(15),
        ],
    )
    .header(header)
    .block(table_block);

    frame.render_widget(table, chunks[0]);

    // 2. Selected Entry Detail View
    let detail_lines = if let Some(selected) = app.audit_entries.get(app.selected_audit_idx) {
        let (decision_str, d_color) = if selected.approved {
            ("Approved / Allowed", Color::Green)
        } else {
            ("Blocked / Denied", Color::Red)
        };

        vec![
            Line::from(vec![
                Span::raw("Actor: "),
                Span::styled(
                    &selected.actor,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  |  Action: "),
                Span::styled(&selected.action, Style::default().fg(Color::Yellow)),
                Span::raw("  |  Decision: "),
                Span::styled(
                    decision_str,
                    Style::default().fg(d_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Target: "),
                Span::styled(&selected.target, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("Policy Reason: "),
                Span::styled(
                    selected
                        .reason
                        .as_deref()
                        .unwrap_or("Standard policy check"),
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ]
    } else {
        vec![Line::from("No entry selected.")]
    };

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .title(" 🔍 Audit Event Inspector ");
    let detail_widget = Paragraph::new(detail_lines).block(detail_block);
    frame.render_widget(detail_widget, chunks[1]);
}
