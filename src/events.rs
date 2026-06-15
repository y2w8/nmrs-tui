use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent};
use nmrs::{SettingsPatch, WifiSecurity};

use crate::{
    action::Action,
    app::{App, Focus, Popups, Selected, Tabs},
    ui::{input::InputMode, toast::Urgency},
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
            KeyCode::Char('q') => app.action.send(Action::Quit),

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
            KeyCode::Char('j') | KeyCode::Down => app.action.send(Action::NextItem(tab)),
            KeyCode::Char('k') | KeyCode::Up => app.action.send(Action::PreviousItem(tab)),

            KeyCode::Char('o') => {
                if tab == Tabs::Devices {
                    app.action.send(Action::TogglePower);
                }
            }

            KeyCode::Char('r') => match tab {
                Tabs::AvailableNetworks | Tabs::KnownNetworks => {
                    app.action.send(Action::Refresh);
                }
                _ => {}
            },

            KeyCode::Enter | KeyCode::Char(' ') => match tab {
                Tabs::KnownNetworks => {
                    if let Some(Selected::Network(net)) = app.selected() {
                        match &app.network_manager.current_network().await {
                            None => {
                                app.action.send(Action::Connect(Box::new((
                                    net.ssid.to_string(),
                                    None,
                                    WifiSecurity::Open,
                                ))));
                            }

                            Some(current) => match current.ssid == net.ssid {
                                true => {
                                    app.action.send(Action::Disconnect);
                                }
                                false => {
                                    app.action.send(Action::Connect(Box::new((
                                        net.ssid.to_string(),
                                        None,
                                        WifiSecurity::Open,
                                    ))));
                                }
                            },
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

            KeyCode::Char('d') => {
                if tab == Tabs::KnownNetworks
                    && let Some(Selected::Network(net)) = app.selected()
                {
                    app.action.send(Action::Forget(net.ssid.to_string()));
                }
            }

            KeyCode::Char('c') => {
                // if tab == Tabs::KnownNetworks
                //     && let Some(Selected::Network(net)) = app.selected()
                //     && let Some(uuid) = app
                //         .network_manager
                //         .nmrs
                //         .get_saved_connection_uuid(&net.ssid)
                //         .await?
                // {
                //     let saved_conn = app.network_manager.nmrs.get_saved_connection(&uuid).await?;
                //     let mut patch = SettingsPatch::default();
                //     patch.autoconnect = Some(!saved_conn.autoconnect);
                //     app.network_manager
                //         .nmrs
                //         .update_saved_connection(&uuid, patch)
                //         .await?;
                //     app.action.send(Action::ShowToast(
                //         None,
                //         format!("{}", !saved_conn.autoconnect).into(),
                //         Urgency::Success,
                //         None,
                //     ));
                //     app.action
                //          .send(Action::SetFocus(Focus::Popup(Popups::EditNetwork)));
                //     app.action.send(Action::SetInputMode(InputMode::Editing));
                // }
                app.action.send(Action::ShowToast(
                    None,
                    "WIP".into(),
                    Urgency::Critical,
                    None,
                ));
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
                    app.action.send(Action::Connect(Box::new((
                        net.ssid.to_string(),
                        None,
                        WifiSecurity::WpaPsk {
                            psk: app.input.value.clone(),
                        },
                    ))));
                    app.action.send(Action::SetInputMode(InputMode::Normal));
                    app.action.send(Action::SetFocus(app.last_focus));
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
                app.action.send(Action::SetInputMode(InputMode::Normal));
                app.action.send(Action::SetFocus(app.last_focus));
                // app.scan.enabled = true;
            }
            _ => {}
        },
    }
    Ok(())
}
