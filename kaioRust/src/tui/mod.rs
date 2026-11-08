pub mod app;
pub mod client;
pub mod event;
pub mod ui;
pub mod views;

use crate::tui::app::TuiApp;
use crate::tui::client::TuiClient;
use crate::tui::event::{handle_key, poll_event, TuiAction};
use anyhow::Result;
use crossterm::event::Event;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{CrosstermBackend, TestBackend};
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;

/// Run the interactive TUI application in full terminal mode.
pub async fn run_tui(mut app: TuiApp, poll_ms: u64, daemon_url: Option<String>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut client = TuiClient::connect(daemon_url.as_deref()).await;
    let _ = client.refresh(&mut app).await;

    let res = run_loop(&mut terminal, &mut app, &mut client, poll_ms).await;

    // Clean up terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut TuiApp,
    client: &mut TuiClient,
    poll_ms: u64,
) -> Result<()> {
    let tick_rate = Duration::from_millis(poll_ms.max(50));

    while !app.should_quit {
        terminal.draw(|frame| {
            ui::render_ui(frame, app);
        })?;

        client.poll_events(app);

        if let Some(Event::Key(key)) = poll_event(tick_rate)? {
            match handle_key(app, key) {
                TuiAction::Quit => break,
                TuiAction::Refresh => {
                    let _ = client.refresh(app).await;
                    app.push_activity("[TUI] Manual refresh triggered");
                }
                TuiAction::ApproveProposal(id) => match client.approve(&id).await {
                    Ok(()) => app.push_activity(format!("[ACTION] Approved proposal {}", id)),
                    Err(error) => app.push_activity(format!("[ACTION] Approve failed: {}", error)),
                },
                TuiAction::RejectProposal(id) => match client.deny(&id).await {
                    Ok(()) => app.push_activity(format!("[ACTION] Denied proposal {}", id)),
                    Err(error) => app.push_activity(format!("[ACTION] Deny failed: {}", error)),
                },
                TuiAction::None => {}
            }
        }
    }

    Ok(())
}

/// Render a single frame snapshot into a String without opening a real terminal.
/// Useful for headless verification, CI testing, and regression analysis.
pub fn render_tui_headless(app: &TuiApp, width: u16, height: u16) -> Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|frame| {
        ui::render_ui(frame, app);
    })?;

    let buffer = terminal.backend().buffer();
    let mut output = String::new();

    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            let cell = &buffer[(x, y)];
            line.push_str(cell.symbol());
        }
        output.push_str(line.trim_end());
        output.push('\n');
    }

    Ok(output)
}
