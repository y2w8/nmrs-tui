use std::{borrow::Cow, time::Duration};

use nmrs::{ConnectionError, Network, WifiDevice, WifiSecurity};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    app::{App, Focus, Tabs},
    ui::{
        input::InputMode,
        toast::{Toast, Urgency},
    },
};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Tick(Duration),
    Refresh,
    SetFocus(Focus),
    SetInputMode(InputMode),

    // Network Commands
    NetworkScanResult(Box<(Vec<Network>, Vec<Network>, Vec<WifiDevice>)>),
    Connect(Box<(String, Option<String>, WifiSecurity)>),
    Forget(String),
    TogglePower,
    Disconnect,

    // UI
    NextItem(Tabs),
    PreviousItem(Tabs),
    ShowToast(
        Option<Cow<'static, str>>,
        Cow<'static, str>,
        Urgency,
        Option<f32>,
    ),
}

pub struct ActionHandler {
    tx: UnboundedSender<Action>,
    rx: UnboundedReceiver<Action>,
}

impl ActionHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Action>();

        Self { tx, rx }
    }

    /// Returns action_tx.
    pub fn sender(&self) -> UnboundedSender<Action> {
        self.tx.clone()
    }

    pub fn send(&self, action: Action) {
        match self.tx.send(action) {
            Ok(_) => {}
            Err(e) => {
                error!("Action failed: {}", e)
            }
        }
    }

    pub async fn handle_actions(app: &mut App) -> anyhow::Result<()> {
        while let Ok(action) = app.action.rx.try_recv() {
            debug!("Action: {:?}", action);

            match action {
                Action::Quit => app.quit(),
                Action::Tick(delta) => {
                    // Toast duration timer
                    app.toasts.iter_mut().for_each(|toast| {
                        toast.duration = toast.duration.saturating_sub(delta);
                    });
                    app.toasts.retain(|toast| !toast.duration.is_zero());

                    // Timers
                    let finished: Vec<Action> = app
                        .timers_mut()
                        .into_iter()
                        .filter(|t| t.enabled)
                        .filter_map(|timer| {
                            timer.tick(delta);
                            if timer.is_finished() {
                                timer.reset();
                                Some(timer.on_finish.clone())
                            } else {
                                None
                            }
                        })
                        .collect();

                    for action in finished {
                        app.action.send(action);
                    }
                }
                Action::Refresh => {
                    if app.scan.enabled {
                        let action_tx = app.action.sender();
                        let network_manager = app.network_manager.clone();
                        tokio::spawn(async move {
                            match tokio::try_join!(
                                network_manager.networks_list(),
                                network_manager.get_devices()
                            ) {
                                Ok(((known, available), devices)) => {
                                    let _ = action_tx.send(Action::NetworkScanResult(Box::new((
                                        known, available, devices,
                                    ))));
                                }
                                Err(_) => {
                                    let _ = action_tx.send(Action::ShowToast(
                                        None,
                                        "Refresh failed!".into(),
                                        Urgency::Critical,
                                        None,
                                    ));
                                }
                            }
                        });
                    }
                }
                Action::SetFocus(new_focus) => {
                    debug!("Set focus to: {:?}!", new_focus);
                    app.last_focus = app.focus;
                    app.focus = new_focus;
                }
                Action::SetInputMode(new_inputmode) => {
                    debug!("Set InputMode to: {:?}!", new_inputmode);
                    app.input.mode = new_inputmode;
                    app.input.value.clear();
                    app.input.reset_cursor();
                }

                // Network
                Action::NetworkScanResult(box_data) => {
                    let (known, available, devices) = *box_data;
                    app.network_manager.current_network =
                        app.network_manager.current_network().await;
                    app.network_manager.current_network_info =
                        if let Some(network) = &app.network_manager.current_network {
                            Some(app.network_manager.show_details(network).await?)
                        } else {
                            None
                        };

                    app.known_networks.set_items(known);
                    app.available_networks.set_items(available);
                    app.devices.set_items(devices);
                }
                Action::Connect(box_data) => {
                    let (ssid, interface, credentials) = *box_data;
                    app.action.send(Action::ShowToast(
                        None,
                        format!("Connecting to {}...", ssid).into(),
                        Urgency::Normal,
                        None,
                    ));

                    let action_tx = app.action.sender();
                    let network_manager = app.network_manager.clone();
                    tokio::spawn(async move {
                        let msg;
                        let urgency;
                        match network_manager.connect(&ssid, interface, credentials).await {
                            Ok(Ok(_)) => {
                                msg = "Connected!";
                                urgency = Urgency::Success;
                            }
                            Ok(Err(ConnectionError::NotFound)) => {
                                msg = "Network not visible — is it in range?";
                                urgency = Urgency::Critical;
                            }
                            Ok(Err(ConnectionError::AuthFailed)) => {
                                let _ = action_tx.send(Action::Forget(ssid.to_string()));
                                msg = "Wrong password!";
                                urgency = Urgency::Critical;
                            }
                            Ok(Err(ConnectionError::Timeout)) => {
                                msg = "Connection timed out — try increasing the timeout";
                                urgency = Urgency::Critical;
                            }
                            Ok(Err(ConnectionError::DhcpFailed)) => {
                                msg = "Failed to get an IP address";
                                urgency = Urgency::Critical;
                            }
                            Ok(Err(_)) => {
                                msg = "Connection failed!";
                                urgency = Urgency::Critical;
                            }
                            Err(_) => {
                                msg = "Operation timed out!";
                                urgency = Urgency::Critical;
                                error!("{}", msg);
                            }
                        }
                        let _ = action_tx.send(Action::ShowToast(None, msg.into(), urgency, None));
                    });
                }
                Action::Forget(ssid) => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        if network_manager.forget(&ssid).await.is_err() {
                            let _ = action_tx.send(Action::ShowToast(
                                None,
                                "Forget failed!".into(),
                                Urgency::Critical,
                                None,
                            ));
                        }
                        let _ = action_tx.send(Action::ShowToast(
                            None,
                            "Forgeted".into(),
                            Urgency::Success,
                            None,
                        ));
                        let _ = action_tx.send(Action::Refresh);
                    });
                }
                Action::TogglePower => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        let enabled = network_manager.nmrs.wifi_state().await.unwrap().enabled;
                        let msg;
                        let urgency;

                        if network_manager
                            .nmrs
                            .set_wireless_enabled(!enabled)
                            .await
                            .is_err()
                        {
                            msg = "Toggle power failed!".to_string();
                            urgency = Urgency::Critical;
                        } else {
                            msg = format!("Wifi {}", if !enabled { "On" } else { "Off" });
                            urgency = Urgency::Success;
                        };
                        let _ = action_tx.send(Action::ShowToast(None, msg.into(), urgency, None));
                        let _ = action_tx.send(Action::Refresh);
                    });
                }
                Action::Disconnect => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        if network_manager.disconnect().await.is_err() {
                            let _ = action_tx.send(Action::ShowToast(
                                None,
                                "Disconnect failed!".into(),
                                Urgency::Critical,
                                None,
                            ));
                        }
                        let _ = action_tx.send(Action::Refresh);
                    });
                }

                // UI
                Action::NextItem(tab) => match tab {
                    Tabs::KnownNetworks => app.known_networks.next(),
                    Tabs::AvailableNetworks => app.available_networks.next(),
                    Tabs::Devices => app.devices.next(),
                },
                Action::PreviousItem(tab) => match tab {
                    Tabs::KnownNetworks => app.known_networks.previous(),
                    Tabs::AvailableNetworks => app.available_networks.previous(),
                    Tabs::Devices => app.devices.previous(),
                },
                Action::ShowToast(title, msg, urgency, duration) => {
                    app.toasts.push(Toast::new(
                        &app.config.ui.toast,
                        title,
                        msg,
                        urgency,
                        duration,
                    ));
                }
            }
        }
        Ok(())
    }
}
