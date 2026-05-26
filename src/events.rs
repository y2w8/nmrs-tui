use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent};
use nmrs::WifiSecurity;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::{App, AppEvent},
    tui::{Selected, Tabs, Tui},
    ui::{input::InputMode, toast::Urgency},
};

pub async fn handle_events(
    app: &mut App,
    tui: &mut Tui,
    key: KeyEvent,
    event_sender: UnboundedSender<AppEvent>,
) -> Result<()> {
    match tui.input.mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => app.quit(),

            // Navigation
            KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
                tui.active_tab = match tui.active_tab {
                    Tabs::KnownNetworks => Tabs::AvailableNetworks,
                    Tabs::AvailableNetworks => Tabs::Devices,
                    Tabs::Devices => Tabs::KnownNetworks,
                };
                tui.update_selected(app);
            }
            KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
                tui.active_tab = match tui.active_tab {
                    Tabs::KnownNetworks => Tabs::Devices,
                    Tabs::AvailableNetworks => Tabs::KnownNetworks,
                    Tabs::Devices => Tabs::AvailableNetworks,
                };
                tui.update_selected(app);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match tui.active_tab {
                    Tabs::Devices => app.devices.next(),
                    Tabs::AvailableNetworks => app.available_networks.next(),
                    Tabs::KnownNetworks => app.known_networks.next(),
                }
                tui.update_selected(app);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match tui.active_tab {
                    Tabs::Devices => app.devices.previous(),
                    Tabs::AvailableNetworks => app.available_networks.previous(),
                    Tabs::KnownNetworks => app.known_networks.previous(),
                }
                tui.update_selected(app);
            }

            KeyCode::Char('r') => match tui.active_tab {
                Tabs::AvailableNetworks | Tabs::KnownNetworks => app.refresh_networks().await?,
                _ => {}
            },
            KeyCode::Enter => match tui.active_tab {
                Tabs::KnownNetworks => {
                    if let Some(Selected::Network(net)) = &tui.selected
                        && app.network_manager.has_saved_connection(&net.ssid).await?
                    {
                        app.network_manager
                            .connect(&net.ssid, None, WifiSecurity::Open)
                            .await;
                        app.known_networks.state.select_first();
                        tui.update_selected(app);
                    }
                }
                Tabs::AvailableNetworks => {
                    if let Some(Selected::Network(net)) = &tui.selected
                        && net.is_psk
                    {
                        tui.input.mode = InputMode::Editing;
                        tui.scan = false;
                    }
                }
                Tabs::Devices => {}
            },
            KeyCode::Char('f') => {
                if tui.active_tab == Tabs::KnownNetworks
                    && let Some(Selected::Network(net)) = &tui.selected
                {
                    let _ = app.event_sender.send(AppEvent::Refresh);
                    app.network_manager.forget(&net.ssid).await?;
                    // tui.update_selected(app);
                }
            }
            _ => {}
        },
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                tui.input.mode = InputMode::Normal;
                if let Some(Selected::Network(net)) = &tui.selected {
                    let _ = event_sender.send(AppEvent::Toast(
                        None,
                        format!("Connecting to {}...", net.ssid).into(),
                        Urgency::Normal,
                        None,
                    ));
                    app.network_manager
                        .connect(
                            &net.ssid,
                            None,
                            WifiSecurity::WpaPsk {
                                psk: tui.input.value.clone(),
                            },
                        )
                        .await;
                    tui.input.value.clear();
                    tui.input.reset_cursor();
                    app.available_networks.next();
                    tui.update_selected(app);
                    tui.scan = true;
                }
            }
            KeyCode::Tab => tui.input.hidden_password = !tui.input.hidden_password,
            KeyCode::Char(to_insert) => tui.input.enter_char(to_insert),
            KeyCode::Backspace => tui.input.delete_char(),
            KeyCode::Left => tui.input.move_cursor_left(),
            KeyCode::Right => tui.input.move_cursor_right(),
            KeyCode::Esc => {
                tui.input.mode = InputMode::Normal;
                tui.input.value.clear();
                tui.input.reset_cursor();
            }
            _ => {}
        },
    }
    Ok(())
}
