use crate::desktop::protocol::PolicyTier;
use crate::tui::app::TuiApp;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render_task(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let has_proposal = app.pending_proposal.is_some();
    let bottom_height = if has_proposal { 8 } else { 3 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(bottom_height),
        ])
        .split(area);

    // 1. Task Header
    let status_color = if has_proposal {
        Color::Yellow
    } else if app.task_title == "Idle" {
        Color::Green
    } else {
        Color::Cyan
    };

    let header_lines = vec![Line::from(vec![
        Span::raw("Active Task: "),
        Span::styled(
            &app.task_title,
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  Status: "),
        Span::styled(
            if has_proposal {
                "Awaiting Approval ⚠️"
            } else {
                "Running / Idle 🟢"
            },
            Style::default().fg(status_color),
        ),
        Span::raw("  |  Approved: "),
        Span::styled(
            format!("{}", app.approved_proposals_count),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  |  Rejected: "),
        Span::styled(
            format!("{}", app.rejected_proposals_count),
            Style::default().fg(Color::Red),
        ),
    ])];

    let header_block = Block::default()
        .borders(Borders::ALL)
        .title(" 🤖 Task Runner ");
    let header_widget = Paragraph::new(header_lines).block(header_block);
    frame.render_widget(header_widget, chunks[0]);

    // 2. Agent Logs Pane
    let log_items: Vec<ListItem> = app
        .agent_logs
        .iter()
        .rev()
        .take(30)
        .rev()
        .map(|line| {
            let color = if line.contains("✅") {
                Color::Green
            } else if line.contains("❌") {
                Color::Red
            } else if line.contains("⚠️") {
                Color::Yellow
            } else if line.contains("THOUGHT:") {
                Color::Magenta
            } else {
                Color::White
            };
            ListItem::new(Span::styled(line, Style::default().fg(color)))
        })
        .collect();

    let logs_block = Block::default()
        .borders(Borders::ALL)
        .title(" 📜 Agent Output Stream ");
    let logs_widget = List::new(log_items).block(logs_block);
    frame.render_widget(logs_widget, chunks[1]);

    // 3. Interactive Tool Proposal Card or Prompt Bar
    if let Some(prop) = &app.pending_proposal {
        let (tier_name, tier_color) = match prop.policy_tier {
            PolicyTier::ReadOnly => ("READ_ONLY", Color::Green),
            PolicyTier::WritePrompt => ("WRITE_PROMPT", Color::Yellow),
            PolicyTier::Dangerous => ("DANGEROUS", Color::Red),
        };

        let card_lines = vec![
            Line::from(vec![
                Span::styled(
                    " TOOL PROPOSAL: ",
                    Style::default()
                        .bg(tier_color)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    &prop.tool_name,
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan),
                ),
                Span::raw("  [Policy Tier: "),
                Span::styled(
                    tier_name,
                    Style::default().fg(tier_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw("]"),
            ]),
            Line::from(vec![
                Span::raw(" Target: "),
                Span::styled(&prop.target, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw(" Details: "),
                Span::styled(&prop.details, Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled(
                    " [y / Enter] Approve Tool Execution ",
                    Style::default()
                        .bg(Color::Green)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled(
                    " [n / Esc] Reject Tool Execution ",
                    Style::default()
                        .bg(Color::Red)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let proposal_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tier_color))
            .title(" ⚠️ Interactive Tool Approval Card ");
        let proposal_widget = Paragraph::new(card_lines).block(proposal_block);
        frame.render_widget(proposal_widget, chunks[2]);
    } else {
        let prompt_lines = vec![Line::from(vec![
            Span::raw("Prompt > "),
            Span::styled(
                "Ready. Dispatched agent tasks will stream here.",
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(" (Press [space] to pause stream, [r] to refresh)"),
        ])];
        let prompt_block = Block::default()
            .borders(Borders::ALL)
            .title(" 💬 Prompt & Status ");
        let prompt_widget = Paragraph::new(prompt_lines).block(prompt_block);
        frame.render_widget(prompt_widget, chunks[2]);
    }
}
