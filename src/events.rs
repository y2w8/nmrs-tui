use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent};
use nmrs::WifiSecurity;

use crate::{
    app::{App, InputMode},
    tui::{Tabs, Tui},
};

pub async fn handle_events(app: &mut App, tui: &mut Tui, key: KeyEvent) -> Result<()> {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => app.quit(),

            // Navigation
            KeyCode::Tab | KeyCode::Char('l') => {
                tui.active_tab = match tui.active_tab {
                    Tabs::KnownNetworks => Tabs::AvailableNetworks,
                    Tabs::AvailableNetworks => Tabs::Devices,
                    Tabs::Devices => Tabs::KnownNetworks,
                };
            }
            KeyCode::BackTab | KeyCode::Char('h') => {
                tui.active_tab = match tui.active_tab {
                    Tabs::KnownNetworks => Tabs::Devices,
                    Tabs::AvailableNetworks => Tabs::KnownNetworks,
                    Tabs::Devices => Tabs::AvailableNetworks,
                };
            }
            KeyCode::Char('j') | KeyCode::Down => match tui.active_tab {
                Tabs::Devices => app.devices.next(),
                Tabs::AvailableNetworks => app.available_networks.next(),
                Tabs::KnownNetworks => app.known_networks.next(),
            },
            KeyCode::Char('k') | KeyCode::Up => match tui.active_tab {
                Tabs::Devices => app.devices.previous(),
                Tabs::AvailableNetworks => app.available_networks.previous(),
                Tabs::KnownNetworks => app.known_networks.previous(),
            },

            KeyCode::Char('r') => match tui.active_tab {
                Tabs::AvailableNetworks | Tabs::KnownNetworks => app.refresh_networks().await?,
                _ => {}
            },
            KeyCode::Enter => match tui.active_tab {
                Tabs::KnownNetworks => {
                    if let Some(network) = Tui::selected_network(&app.known_networks) {
                        tui.selected_network = Some(network.clone());
                        if app
                            .network_manager
                            .has_saved_connection(&network.ssid)
                            .await?
                        {
                            app.network_manager
                                .connect(&network.ssid, None, WifiSecurity::Open)
                                .await?;
                        }
                    }
                }
                Tabs::AvailableNetworks => {
                    if let Some(network) = Tui::selected_network(&app.available_networks) {
                        tui.selected_network = Some(network.clone());
                        if network.is_psk {
                            app.password_input.clear();
                            app.input_mode = InputMode::Editing
                        }
                    }
                }
                Tabs::Devices => {}
            },
            KeyCode::Char('f') => {
                if tui.active_tab == Tabs::KnownNetworks {
                    tui.selected_network = Tui::selected_network(&app.known_networks);
                    app.network_manager
                        .forget(&tui.selected_network.clone().unwrap().ssid)
                        .await?;
                }
            }
            _ => {}
        },
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                app.input_mode = InputMode::Normal;
                app.network_manager
                    .connect(
                        &tui.selected_network.clone().unwrap().ssid,
                        None,
                        WifiSecurity::WpaPsk {
                            psk: tui.input.value.clone(),
                        },
                    )
                    .await?;
                app.password_input.clear();
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.password_input.push(c);
            }
            KeyCode::Backspace => {
                app.password_input.pop();
            }
            _ => {}
        },
    }
    Ok(())
}
