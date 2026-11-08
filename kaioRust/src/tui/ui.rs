use crate::tui::app::{ActiveTab, DaemonConnectionState, TuiApp};
use crate::tui::views::{
    audit::render_audit, dashboard::render_dashboard, memory::render_memory, review::render_review,
    task::render_task,
};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};
use ratatui::Frame;

pub fn render_ui(frame: &mut Frame, app: &TuiApp) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & Tabs
            Constraint::Min(10),   // Active Workspace View
            Constraint::Length(1), // Footer / Keybindings
        ])
        .split(size);

    render_header_and_tabs(frame, app, chunks[0]);

    match app.active_tab {
        ActiveTab::Dashboard => render_dashboard(frame, app, chunks[1]),
        ActiveTab::Memory => render_memory(frame, app, chunks[1]),
        ActiveTab::Task => render_task(frame, app, chunks[1]),
        ActiveTab::Review => render_review(frame, app, chunks[1]),
        ActiveTab::Audit => render_audit(frame, app, chunks[1]),
    }

    render_footer(frame, app, chunks[2]);

    if app.show_help {
        render_help_modal(frame, size);
    }
}

fn render_header_and_tabs(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(area);

    let daemon_span = match app.daemon_state {
        DaemonConnectionState::Connected => {
            Span::styled("[Daemon: Connected 🟢]", Style::default().fg(Color::Green))
        }
        DaemonConnectionState::Disconnected => {
            Span::styled("[Daemon: Offline 🔴]", Style::default().fg(Color::Red))
        }
        DaemonConnectionState::Embedded => {
            Span::styled("[Daemon: Embedded 📦]", Style::default().fg(Color::Cyan))
        }
    };

    let title_line = Line::from(vec![
        Span::styled(
            " Kaioken v0.1.0 ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        ),
        Span::raw("── "),
        Span::styled(
            format!("Repo: {}/{} ", app.repo_name, app.branch),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("── "),
        daemon_span,
        Span::raw(" ── "),
        Span::styled(
            format!("[Sync: {}]", app.sync_time),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let title_widget = Paragraph::new(title_line).alignment(Alignment::Left);
    frame.render_widget(title_widget, chunks[0]);

    let tab_titles: Vec<Line> = ActiveTab::all()
        .iter()
        .map(|t| Line::from(Span::raw(t.title())))
        .collect();

    let tabs = Tabs::new(tab_titles)
        .select(app.active_tab.to_index())
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(" | "));

    frame.render_widget(tabs, chunks[1]);
}

fn render_footer(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let mode_str = if app.read_only { " [READ-ONLY] " } else { "" };

    let footer_line = Line::from(vec![
        Span::styled(
            " [Tab] Switch  [1..5] Jump  [↑/↓] Scroll  [y/n] Approve/Reject  [space] Pause  [r] Refresh  [?] Help  [q] Quit ",
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
        Span::styled(mode_str, Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
    ]);

    let footer_widget = Paragraph::new(footer_line).alignment(Alignment::Center);
    frame.render_widget(footer_widget, area);
}

fn render_help_modal(frame: &mut Frame, area: Rect) {
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 16.min(area.height.saturating_sub(2));

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "Kaioken Interactive TUI - Keybindings",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab / Shift+Tab  ", Style::default().fg(Color::Cyan)),
            Span::raw(": Cycle through workspace tabs"),
        ]),
        Line::from(vec![
            Span::styled("  1 .. 5           ", Style::default().fg(Color::Cyan)),
            Span::raw(": Jump directly to tab (Dashboard, Memory, Task, Review, Audit)"),
        ]),
        Line::from(vec![
            Span::styled("  ↑ / ↓ (Up/Down)  ", Style::default().fg(Color::Cyan)),
            Span::raw(": Scroll lists, diffs, and audit log entries"),
        ]),
        Line::from(vec![
            Span::styled("  y / Enter        ", Style::default().fg(Color::Green)),
            Span::raw(": Approve pending tool execution in Task Runner"),
        ]),
        Line::from(vec![
            Span::styled("  n / Esc          ", Style::default().fg(Color::Red)),
            Span::raw(": Reject pending tool execution"),
        ]),
        Line::from(vec![
            Span::styled("  Space            ", Style::default().fg(Color::Cyan)),
            Span::raw(": Pause / resume real-time activity stream"),
        ]),
        Line::from(vec![
            Span::styled("  r                ", Style::default().fg(Color::Cyan)),
            Span::raw(": Manually trigger database / daemon refresh"),
        ]),
        Line::from(vec![
            Span::styled("  ?                ", Style::default().fg(Color::Cyan)),
            Span::raw(": Toggle this help dialog"),
        ]),
        Line::from(vec![
            Span::styled("  q / Ctrl+C       ", Style::default().fg(Color::Yellow)),
            Span::raw(": Exit Kaioken TUI"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press [?] or [Esc] to close this help modal.",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" ❓ Keyboard Shortcuts & Help ");
    let paragraph = Paragraph::new(help_text).block(block);
    frame.render_widget(paragraph, popup_area);
}
