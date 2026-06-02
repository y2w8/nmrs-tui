use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent};
use nmrs::WifiSecurity;

use crate::{
    action::Actions,
    app::{App, Focus, Popups, Selected, Tabs},
    ui::input::InputMode,
};

pub async fn handle_events(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.focus {
        Focus::Tab(tab) => Ok(handle_tabs(app, key, tab).await?),
        Focus::Popup(popup) => Ok(handle_popups(app, key, popup).await?),
    }
}

async fn handle_tabs(app: &mut App, key: KeyEvent, tab: Tabs) -> Result<()> {
    match app.input.mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => app.action.send(Actions::Quit),

            // Navigation
            KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
                app.focus = match tab {
                    Tabs::KnownNetworks => Focus::Tab(Tabs::AvailableNetworks),
                    Tabs::AvailableNetworks => Focus::Tab(Tabs::Devices),
                    Tabs::Devices => Focus::Tab(Tabs::KnownNetworks),
                };
            }
            KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
                app.focus = match tab {
                    Tabs::KnownNetworks => Focus::Tab(Tabs::Devices),
                    Tabs::AvailableNetworks => Focus::Tab(Tabs::KnownNetworks),
                    Tabs::Devices => Focus::Tab(Tabs::AvailableNetworks),
                };
            }
            KeyCode::Char('j') | KeyCode::Down => app.action.send(Actions::NextItem(tab)),
            KeyCode::Char('k') | KeyCode::Up => app.action.send(Actions::PreviousItem(tab)),

            KeyCode::Char('o') => {
                if tab == Tabs::Devices {
                    app.action.send(Actions::TogglePower);
                }
            }

            KeyCode::Char('r') => match tab {
                Tabs::AvailableNetworks | Tabs::KnownNetworks => {
                    app.action.send(Actions::Refresh);
                }
                _ => {}
            },

            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Focus::Tab(tab) = app.focus {
                    match tab {
                        Tabs::KnownNetworks => {
                            if let Some(Selected::Network(net)) = app.selected() {
                                let is_saved =
                                    app.network_manager.has_saved_connection(&net.ssid).await?;
                                if is_saved {
                                    match &app.network_manager.current_network().await {
                                        None => {
                                            app.action.send(Actions::Connect(
                                                net.ssid.to_string(),
                                                None,
                                                WifiSecurity::Open,
                                            ));
                                        }

                                        Some(current) => {
                                            if current.ssid == net.ssid {
                                                app.action.send(Actions::Disconnect);
                                            } else {
                                                app.action.send(Actions::Connect(
                                                    net.ssid.to_string(),
                                                    None,
                                                    WifiSecurity::Open,
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Tabs::AvailableNetworks => {
                            if let Some(Selected::Network(net)) = app.selected()
                                && net.is_psk
                            {
                                app.action
                                    .send(Actions::SetFocus(Focus::Popup(Popups::Password)));
                                app.action.send(Actions::SetInputMode(InputMode::Editing));
                            }
                        }
                        Tabs::Devices => {}
                    }
                }
            }

            KeyCode::Char('f') => {
                if let Focus::Tab(tab) = app.focus
                    && tab == Tabs::KnownNetworks
                    && let Some(Selected::Network(net)) = app.selected()
                {
                    app.action.send(Actions::Forget(net.ssid.to_string()));
                }
            }
            _ => {}
        },
        InputMode::Editing => {}
    }
    Ok(())
}

async fn handle_popups(app: &mut App, key: KeyEvent, popup: Popups) -> Result<()> {
    match app.input.mode {
        InputMode::Normal => {}
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                if let Some(Selected::Network(net)) = app.selected() {
                    app.action.send(Actions::Connect(
                        net.ssid.to_string(),
                        None,
                        WifiSecurity::WpaPsk {
                            psk: app.input.value.clone(),
                        },
                    ));
                    app.action.send(Actions::SetInputMode(InputMode::Normal));
                    app.action.send(Actions::SetFocus(app.last_focus));
                    app.scan.enabled = true;
                }
            }
            KeyCode::Tab => {
                if popup == Popups::Password {
                    app.input.hidden_password = !app.input.hidden_password
                }
            }
            KeyCode::Char(to_insert) => app.input.enter_char(to_insert),
            KeyCode::Backspace => app.input.delete_char(),
            KeyCode::Left => app.input.move_cursor_left(),
            KeyCode::Right => app.input.move_cursor_right(),
            KeyCode::Esc => {
                app.action.send(Actions::SetInputMode(InputMode::Normal));
                app.action.send(Actions::SetFocus(app.last_focus));
                // app.scan.enabled = true;
            }
            _ => {}
        },
    }
    Ok(())
}
