use crate::tui::app::TuiApp;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render_memory(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let filtered = app.filtered_symbols();

    // 1. Symbol List Pane
    let items: Vec<ListItem> = if filtered.is_empty() {
        vec![ListItem::new("No symbols found in memory database.")]
    } else {
        filtered
            .iter()
            .enumerate()
            .map(|(idx, sym)| {
                let is_selected = idx == app.selected_symbol_idx;
                let kind_color = match sym.kind.to_lowercase().as_str() {
                    "function" | "fn" | "method" => Color::Green,
                    "struct" => Color::Cyan,
                    "enum" => Color::Yellow,
                    "trait" | "interface" => Color::Magenta,
                    _ => Color::White,
                };

                let prefix = if is_selected { "► " } else { "  " };

                let style = if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw(prefix),
                        Span::styled(format!("[{}] ", sym.kind), Style::default().fg(kind_color)),
                        Span::styled(&sym.name, Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            format!("{}:{}", sym.file_path, sym.line_number),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                ])
                .style(style)
            })
            .collect()
    };

    let title = if app.symbol_search_query.is_empty() {
        format!(" 🕸️ Symbols & Files ({}) ", filtered.len())
    } else {
        format!(
            " 🕸️ Symbols & Files (Filtered: {} for '{}') ",
            filtered.len(),
            app.symbol_search_query
        )
    };

    let list_block = Block::default().borders(Borders::ALL).title(title);
    let list_widget = List::new(items).block(list_block);
    frame.render_widget(list_widget, chunks[0]);

    // 2. Symbol Details Pane
    let detail_lines = if let Some(selected) = filtered.get(app.selected_symbol_idx) {
        vec![
            Line::from(vec![
                Span::raw("Symbol:       "),
                Span::styled(
                    &selected.name,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Kind:         "),
                Span::styled(&selected.kind, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("File Path:    "),
                Span::styled(&selected.file_path, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("Line:         "),
                Span::styled(
                    format!("{}", selected.line_number),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::raw("Visibility:   "),
                Span::styled(&selected.visibility, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Callers:      "),
                Span::styled(
                    format!("{}", selected.callers_count),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Documentation Mirror (Level 0):",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from(Span::styled(
                selected
                    .doc_summary
                    .as_deref()
                    .unwrap_or("No documentation facts generated yet for this symbol."),
                Style::default().fg(Color::Gray),
            )),
        ]
    } else {
        vec![Line::from(
            "Select a symbol to inspect its details and call graph.",
        )]
    };

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .title(" 🔍 Symbol Details & Call Graph ");
    let detail_widget = Paragraph::new(detail_lines).block(detail_block);
    frame.render_widget(detail_widget, chunks[1]);
}
