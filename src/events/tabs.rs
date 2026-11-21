use crossterm::event::{KeyCode, KeyEvent};
use nmrs::WifiSecurity;

use crate::{
    action::{Action, ConnectRequest},
    app::{App, Focus, Selected},
    ui::{input::Input, popup::Popups, tabs::Tabs},
};

pub fn handle(app: &mut App, key: KeyEvent, tab: Tabs) {
    match key.code {
        KeyCode::Char('q') => app.action.send(Action::Quit),

        // Navigation
        KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
            app.action.send(Action::SetFocus(Focus::Tab(tab.next())));
        }
        KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
            app.action.send(Action::SetFocus(Focus::Tab(tab.prev())));
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
                    Popups::Password(Input::new(), Box::new(net)).open(&app.action);
                }
            }
            Tabs::Devices => {}
        },

        _ => {}
    }
}
