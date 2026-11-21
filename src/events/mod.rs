use crate::app::{App, Focus};
use crossterm::event::{KeyEvent, KeyEventKind};

mod popups;
mod tabs;

pub fn handle_events(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    match app.focus {
        Focus::Tab(tab) => tabs::handle(app, key, tab),
        Focus::Popup(_) => popups::handle(app, key),
    }
}
