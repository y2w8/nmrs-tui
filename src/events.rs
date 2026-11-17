use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use nmrs::WifiSecurity;

use crate::{
    action::{Action, ConnectRequest},
    app::{App, Focus, Popups, Selected, Tabs},
    ui::input::InputMode,
};

pub fn handle_events(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    match app.focus {
        Focus::Tab(tab) => handle_tabs(app, key, tab),
        Focus::Popup(popup) => handle_popups(app, key, popup),
    }
}

fn handle_tabs(app: &mut App, key: KeyEvent, tab: Tabs) {
    if app.input.mode != InputMode::Normal {
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.action.send(Action::Quit),

        // Navigation
        KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
            app.focus = Focus::Tab(tab.next());
        }
        KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
            app.focus = Focus::Tab(tab.prev());
        }
        KeyCode::Char('j') | KeyCode::Down => app.action.send(Action::NextItem(tab)),
        KeyCode::Char('k') | KeyCode::Up => app.action.send(Action::PreviousItem(tab)),

        // Global / Tab Actions
        KeyCode::Char('r') => app.action.send(Action::Refresh),
        KeyCode::Char('a') if tab == Tabs::Devices => {
            app.action.send(Action::ToggleAirplaneMode);
        }
        KeyCode::Char('o') if tab == Tabs::Devices => {
            app.action.send(Action::TogglePower);
        }
        KeyCode::Char('d') if tab == Tabs::KnownNetworks => {
            if let Some(Selected::Network(net)) = app.selected() {
                app.action.send(Action::Forget { ssid: net.ssid });
            }
        }
        KeyCode::Char('t') if tab == Tabs::KnownNetworks => {
            if let Some(Selected::Network(net)) = app.selected() {
                app.action
                    .send(Action::ToggleAutoconnect { ssid: net.ssid });
            }
        }

        KeyCode::Enter | KeyCode::Char(' ') => match tab {
            Tabs::KnownNetworks => {
                if let Some(Selected::Network(net)) = app.selected() {
                    let is_connected = app
                        .network_manager
                        .current_network
                        .as_ref()
                        .is_some_and(|current| current.ssid == net.ssid);

                    if is_connected {
                        app.action.send(Action::Disconnect);
                    } else {
                        app.action.send(Action::Connect(Box::new(ConnectRequest {
                            ssid: net.ssid,
                            interface: None,
                            credentials: WifiSecurity::Open,
                        })));
                    }
                }
            }
            Tabs::AvailableNetworks => {
                if let Some(Selected::Network(net)) = app.selected()
                    && net.is_psk
                {
                    app.action
                        .send(Action::SetFocus(Focus::Popup(Popups::Password)));
                    app.action.send(Action::SetInputMode(InputMode::Editing));
                }
            }
            Tabs::Devices => {}
        },

        _ => {}
    }
}

fn handle_popups(app: &mut App, key: KeyEvent, popup: Popups) {
    if app.input.mode != InputMode::Editing {
        return;
    }

    match key.code {
        KeyCode::Enter => {
            if let Some(Selected::Network(net)) = app.selected() {
                app.action.send(Action::Connect(Box::new(ConnectRequest {
                    ssid: net.ssid,
                    interface: None,
                    credentials: WifiSecurity::WpaPsk {
                        psk: app.input.value.clone(),
                    },
                })));
                app.action.send(Action::SetInputMode(InputMode::Normal));
                app.action.send(Action::SetFocus(app.last_focus));
            }
        }
        KeyCode::Tab if popup == Popups::Password => {
            app.input.hidden_password = !app.input.hidden_password;
        }
        KeyCode::Char(to_insert) => app.input.enter_char(to_insert),
        KeyCode::Backspace => app.input.delete_char(),
        KeyCode::Left => app.input.move_cursor_left(),
        KeyCode::Right => app.input.move_cursor_right(),
        KeyCode::Esc => {
            app.action.send(Action::SetInputMode(InputMode::Normal));
            app.action.send(Action::SetFocus(app.last_focus));
        }
        _ => {}
    }
}
