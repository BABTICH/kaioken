use crate::tui::app::TuiApp;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render_review(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(8)])
        .split(area);

    // 1. Risk Assessment Top Card
    let risk_color = match app.risk_level.to_lowercase().as_str() {
        "critical" | "high" => Color::Red,
        "medium" => Color::Yellow,
        "low" => Color::Green,
        _ => Color::Cyan,
    };

    let mut assessment_lines = vec![Line::from(vec![
        Span::raw("Risk Score:      "),
        Span::styled(
            format!("{}/100", app.risk_score),
            Style::default().fg(risk_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  Risk Level: "),
        Span::styled(
            &app.risk_level,
            Style::default()
                .bg(risk_color)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  Breaking Change Warnings: "),
        Span::styled(
            format!("{}", app.risk_reasons.len()),
            Style::default().fg(Color::White),
        ),
    ])];

    if app.risk_reasons.is_empty() {
        assessment_lines.push(Line::from(Span::styled(
            "No breaking signature changes or critical risk factors detected.",
            Style::default().fg(Color::Green),
        )));
    } else {
        for reason in app.risk_reasons.iter().take(3) {
            assessment_lines.push(Line::from(Span::styled(
                reason,
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    let assess_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(risk_color))
        .title(" 🛡️ Pre-Commit Risk Inspector ");
    let assess_widget = Paragraph::new(assessment_lines).block(assess_block);
    frame.render_widget(assess_widget, chunks[0]);

    // 2. Diff Browser Pane
    let diff_lines: Vec<Line> = app
        .diff_content
        .lines()
        .skip(app.review_scroll)
        .map(|line| {
            if line.starts_with('+') && !line.starts_with("+++") {
                Line::from(Span::styled(line, Style::default().fg(Color::Green)))
            } else if line.starts_with('-') && !line.starts_with("---") {
                Line::from(Span::styled(line, Style::default().fg(Color::Red)))
            } else if line.starts_with("@@") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("diff --git") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(line, Style::default().fg(Color::Gray)))
            }
        })
        .collect();

    let diff_title = format!(
        " 📝 Git Diff Viewer (Scroll: Line {}) [↑/↓ to scroll] ",
        app.review_scroll
    );
    let diff_block = Block::default().borders(Borders::ALL).title(diff_title);
    let diff_widget = Paragraph::new(diff_lines).block(diff_block);
    frame.render_widget(diff_widget, chunks[1]);
}
