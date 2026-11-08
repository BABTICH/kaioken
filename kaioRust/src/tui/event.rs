use crate::tui::app::{ActiveTab, TuiApp};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiAction {
    None,
    Quit,
    Refresh,
    ApproveProposal(String),
    RejectProposal(String),
}

pub fn handle_key(app: &mut TuiApp, key: KeyEvent) -> TuiAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return TuiAction::Quit;
    }

    if app.show_help {
        if key.code == KeyCode::Char('?')
            || key.code == KeyCode::Esc
            || key.code == KeyCode::Char('q')
        {
            app.show_help = false;
        }
        return TuiAction::None;
    }

    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            TuiAction::Quit
        }
        KeyCode::Tab => {
            app.next_tab();
            TuiAction::None
        }
        KeyCode::BackTab => {
            app.prev_tab();
            TuiAction::None
        }
        KeyCode::Char('1') => {
            app.set_tab(ActiveTab::Dashboard);
            TuiAction::None
        }
        KeyCode::Char('2') => {
            app.set_tab(ActiveTab::Memory);
            TuiAction::None
        }
        KeyCode::Char('3') => {
            app.set_tab(ActiveTab::Task);
            TuiAction::None
        }
        KeyCode::Char('4') => {
            app.set_tab(ActiveTab::Review);
            TuiAction::None
        }
        KeyCode::Char('5') => {
            app.set_tab(ActiveTab::Audit);
            TuiAction::None
        }
        KeyCode::Up => {
            app.scroll_up();
            TuiAction::None
        }
        KeyCode::Down => {
            app.scroll_down();
            TuiAction::None
        }
        KeyCode::Char(' ') => {
            app.toggle_pause();
            TuiAction::None
        }
        KeyCode::Char('?') => {
            app.toggle_help();
            TuiAction::None
        }
        KeyCode::Char('r') => TuiAction::Refresh,
        KeyCode::Char('y') | KeyCode::Enter => {
            if !app.read_only {
                if let Some(id) = app.approve_pending_proposal() {
                    TuiAction::ApproveProposal(id)
                } else {
                    TuiAction::None
                }
            } else {
                app.push_activity("[WARN] Cannot approve tool execution in read-only mode");
                TuiAction::None
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            if !app.read_only {
                if let Some(id) = app.reject_pending_proposal() {
                    TuiAction::RejectProposal(id)
                } else {
                    TuiAction::None
                }
            } else {
                TuiAction::None
            }
        }
        _ => TuiAction::None,
    }
}

pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
