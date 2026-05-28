use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent};
use nmrs::{Network, WifiSecurity};

use crate::{
    app::{App, AppEvent},
    tui::{Focus, Popups, Selected, Tabs, Tui},
    ui::{input::InputMode, toast::Urgency},
};

pub async fn handle_events(app: &mut App, tui: &mut Tui, key: KeyEvent) -> Result<()> {
    match tui.focus {
        Focus::Tab(tab) => Ok(handle_tabs(app, tui, key, tab).await?),
        Focus::Popup(popup) => Ok(handle_popups(app, tui, key, popup).await?),
    }
}

async fn handle_tabs(app: &mut App, tui: &mut Tui, key: KeyEvent, tab: Tabs) -> Result<()> {
    match tui.input.mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => app.quit(),

            // Navigation
            KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
                tui.focus = match tab {
                    Tabs::KnownNetworks => Focus::Tab(Tabs::AvailableNetworks),
                    Tabs::AvailableNetworks => Focus::Tab(Tabs::Devices),
                    Tabs::Devices => Focus::Tab(Tabs::KnownNetworks),
                };
                tui.update_selected(app);
            }
            KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
                tui.focus = match tab {
                    Tabs::KnownNetworks => Focus::Tab(Tabs::Devices),
                    Tabs::AvailableNetworks => Focus::Tab(Tabs::KnownNetworks),
                    Tabs::Devices => Focus::Tab(Tabs::AvailableNetworks),
                };
                tui.update_selected(app);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match tab {
                    Tabs::Devices => app.devices.next(),
                    Tabs::AvailableNetworks => app.available_networks.next(),
                    Tabs::KnownNetworks => app.known_networks.next(),
                }
                tui.update_selected(app);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match tab {
                    Tabs::Devices => app.devices.previous(),
                    Tabs::AvailableNetworks => app.available_networks.previous(),
                    Tabs::KnownNetworks => app.known_networks.previous(),
                }
                tui.update_selected(app);
            }

            KeyCode::Char('o') => {
                if tab == Tabs::Devices {
                    app.event_sender.send(AppEvent::Refresh)?;
                    app.network_manager.toggle_network().await?;
                }
            }

            KeyCode::Char('r') => match tab {
                Tabs::AvailableNetworks | Tabs::KnownNetworks => {
                    app.event_sender.send(AppEvent::Refresh)?;
                }
                _ => {}
            },

            KeyCode::Enter => {
                if let Focus::Tab(tab) = tui.focus {
                    match tab {
                        Tabs::KnownNetworks => {
                            if let Some(Selected::Network(net)) = &tui.selected {
                                if app.network_manager.has_saved_connection(&net.ssid).await?
                                    && app.network_manager.current_connection.is_none()
                                {
                                    app.network_manager
                                        .connect(&net.ssid, None, WifiSecurity::Open)
                                        .await;
                                    app.known_networks.state.select_first();
                                    tui.update_selected(app);
                                } else if let Some(current_connection) =
                                    &app.network_manager.current_connection
                                    && current_connection.ssid == net.ssid
                                {
                                    app.network_manager.nmrs.disconnect(None).await?
                                }
                            }
                        }
                        Tabs::AvailableNetworks => {
                            if let Some(Selected::Network(net)) = &tui.selected
                                && net.is_psk
                            {
                                tui.change_focus(Focus::Popup(Popups::Password));
                                tui.input.mode = InputMode::Editing;
                                tui.scan = false;
                            }
                        }
                        Tabs::Devices => {}
                    }
                }
            }

            KeyCode::Char('f') => {
                if let Focus::Tab(tab) = tui.focus
                    && tab == Tabs::KnownNetworks
                    && let Some(Selected::Network(net)) = &tui.selected
                {
                    app.event_sender.send(AppEvent::Refresh)?;
                    app.network_manager.forget(&net.ssid).await?;
                    // tui.update_selected(app);
                }
            }
            _ => {}
        },
        InputMode::Editing => {}
    }
    Ok(())
}

async fn handle_popups(app: &mut App, tui: &mut Tui, key: KeyEvent, popup: Popups) -> Result<()> {
    match tui.input.mode {
        InputMode::Normal => {}
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                if let Some(Selected::Network(net)) = &tui.selected {
                    app.event_sender.send(AppEvent::Toast(
                        None,
                        format!("Connecting to {}...", net.ssid).into(),
                        Urgency::Normal,
                        None,
                    ))?;
                    app.network_manager
                        .connect(
                            &net.ssid,
                            None,
                            WifiSecurity::WpaPsk {
                                psk: tui.input.value.clone(),
                            },
                        )
                        .await;
                    app.available_networks.next();
                    tui.update_selected(app);
                    tui.input.change_mode(InputMode::Normal);
                    tui.change_focus(tui.last_focus);
                    tui.scan = true;
                }
            }
            KeyCode::Tab => {
                if popup == Popups::Password {
                    tui.input.hidden_password = !tui.input.hidden_password
                }
            }
            KeyCode::Char(to_insert) => tui.input.enter_char(to_insert),
            KeyCode::Backspace => tui.input.delete_char(),
            KeyCode::Left => tui.input.move_cursor_left(),
            KeyCode::Right => tui.input.move_cursor_right(),
            KeyCode::Esc => {
                tui.input.change_mode(InputMode::Normal);
                tui.change_focus(tui.last_focus);
                tui.scan = true;
            }
            _ => {}
        },
    }
    Ok(())
}
