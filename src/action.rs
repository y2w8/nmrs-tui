use std::{borrow::Cow, time::Duration};

use nmrs::{ConnectionError, Network, WifiDevice, WifiSecurity};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    app::{App, Focus, Popups, Tabs},
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
    NetworkScanResult(Box<NetworkScanResult>),
    Connect(Box<ConnectRequest>),
    Forget { ssid: String },
    ToggleAirplaneMode,
    TogglePower,
    Disconnect,

    // UI
    NextItem(Tabs),
    PreviousItem(Tabs),
    ShowToast(Box<ToastRequest>),
}

#[derive(Debug, Clone)]
pub struct NetworkScanResult {
    pub known: Vec<Network>,
    pub available: Vec<Network>,
    pub devices: Vec<WifiDevice>,
}

#[derive(Debug, Clone)]
pub struct ConnectRequest {
    pub ssid: String,
    pub interface: Option<String>,
    pub credentials: WifiSecurity,
}

#[derive(Debug, Clone)]
pub struct ToastRequest {
    pub title: Option<Cow<'static, str>>,
    pub msg: Cow<'static, str>,
    pub urgency: Urgency,
    pub duration: Option<f32>,
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
                                    _ = action_tx.send(Action::NetworkScanResult(Box::new(
                                        NetworkScanResult {
                                            known,
                                            available,
                                            devices,
                                        },
                                    )));
                                }
                                Err(_) => {
                                    _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                                        title: None,
                                        msg: "Refresh failed!".into(),
                                        urgency: Urgency::Critical,
                                        duration: None,
                                    })));
                                }
                            }
                        });
                    }
                }
                Action::SetFocus(new_focus) => {
                    debug!("Set focus to: {:?}!", new_focus);
                    app.last_focus = app.focus;
                    app.focus = new_focus;
                    match new_focus {
                        Focus::Popup(popup) => {
                            if popup == Popups::Password {
                                app.scan.disable();
                                debug!("Scan disabled (password popup open)");
                            } else {
                                app.scan.enable();
                            }
                        }
                        Focus::Tab(_) => app.scan.enable(),
                    }
                }
                Action::SetInputMode(new_inputmode) => {
                    debug!("Set InputMode to: {:?}!", new_inputmode);
                    app.input.mode = new_inputmode;
                    app.input.value.clear();
                    app.input.reset_cursor();
                }

                // Network
                Action::NetworkScanResult(box_data) => {
                    let scan_result = *box_data;
                    app.network_manager.current_network =
                        app.network_manager.current_network().await;
                    app.network_manager.current_network_info =
                        if let Some(network) = &app.network_manager.current_network {
                            match app.network_manager.show_details(network).await {
                                Ok(conn_info) => Some(conn_info),
                                Err(_) => {
                                    app.action.send(Action::ShowToast(Box::new(ToastRequest {
                                        title: None,
                                        msg: "Failed to get current network info!".into(),
                                        urgency: Urgency::Critical,
                                        duration: None,
                                    })));
                                    None
                                }
                            }
                        } else {
                            None
                        };

                    app.known_networks.set_items(scan_result.known);
                    app.available_networks.set_items(scan_result.available);
                    app.devices.set_items(scan_result.devices);
                }
                Action::Connect(box_data) => {
                    let conn_req = *box_data;
                    app.action.send(Action::ShowToast(Box::new(ToastRequest {
                        title: None,
                        msg: format!("Connecting to {}...", conn_req.ssid).into(),
                        urgency: Urgency::Normal,
                        duration: None,
                    })));

                    let action_tx = app.action.sender();
                    let network_manager = app.network_manager.clone();
                    tokio::spawn(async move {
                        let msg;
                        let urgency;
                        match network_manager
                            .connect(&conn_req.ssid, conn_req.interface, conn_req.credentials)
                            .await
                        {
                            Ok(Ok(_)) => {
                                msg = "Connected!";
                                urgency = Urgency::Success;
                            }
                            Ok(Err(ConnectionError::NotFound)) => {
                                msg = "Network not visible — is it in range?";
                                urgency = Urgency::Critical;
                            }
                            Ok(Err(ConnectionError::AuthFailed)) => {
                                _ = action_tx.send(Action::Forget {
                                    ssid: conn_req.ssid.to_string(),
                                });
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
                        _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                            title: None,
                            msg: msg.into(),
                            urgency,
                            duration: None,
                        })));
                    });
                }
                Action::Forget { ssid } => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        match network_manager.forget(&ssid).await {
                            Ok(_) => {
                                _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                                    title: None,
                                    msg: "Forgotten!".into(),
                                    urgency: Urgency::Success,
                                    duration: None,
                                })));
                            }
                            Err(_) => {
                                _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                                    title: None,
                                    msg: "Forget failed!".into(),
                                    urgency: Urgency::Critical,
                                    duration: None,
                                })));
                            }
                        }
                        _ = action_tx.send(Action::Refresh);
                    });
                }
                Action::ToggleAirplaneMode => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        let enabled = match network_manager.airplane_mode_state().await {
                            Ok(state) => state.is_airplane_mode(),
                            Err(_) => {
                                _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                                    title: None,
                                    msg: "Failed to get airplane mode state!".into(),
                                    urgency: Urgency::Critical,
                                    duration: None,
                                })));
                                return;
                            }
                        };

                        let msg;
                        let urgency;

                        match network_manager.set_airplane_mode(!enabled).await {
                            Ok(_) => {
                                debug!("Airplane set to {}", !enabled);
                                msg = format!(
                                    "Airplane mode {}",
                                    if !enabled { "On" } else { "Off" }
                                );
                                urgency = Urgency::Success;
                            }
                            Err(_) => {
                                msg = "Toggle airplane mode failed!".to_string();
                                urgency = Urgency::Critical;
                            }
                        };
                        _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                            title: None,
                            msg: msg.into(),
                            urgency,
                            duration: None,
                        })));
                        _ = action_tx.send(Action::Refresh);
                    });
                }
                Action::TogglePower => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        let enabled = match network_manager.wifi_state().await {
                            Ok(wifi_state) => wifi_state.enabled,
                            Err(_) => {
                                _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                                    title: None,
                                    msg: "Failed to get wifi state!".into(),
                                    urgency: Urgency::Critical,
                                    duration: None,
                                })));
                                return;
                            }
                        };
                        let msg;
                        let urgency;

                        match network_manager.set_wireless_enabled(!enabled).await {
                            Ok(_) => {
                                debug!("Wifi radio set to {}", !enabled);
                                msg = format!("Wifi {}", if !enabled { "On" } else { "Off" });
                                urgency = Urgency::Success;
                            }
                            Err(_) => {
                                msg = "Toggle power failed!".to_string();
                                urgency = Urgency::Critical;
                            }
                        };
                        _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                            title: None,
                            msg: msg.into(),
                            urgency,
                            duration: None,
                        })));
                        _ = action_tx.send(Action::Refresh);
                    });
                }
                Action::Disconnect => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        match network_manager.disconnect().await {
                            Ok(_) => {
                                _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                                    title: None,
                                    msg: "Disconnected!".into(),
                                    urgency: Urgency::Success,
                                    duration: None,
                                })));
                            }
                            Err(_) => {
                                _ = action_tx.send(Action::ShowToast(Box::new(ToastRequest {
                                    title: None,
                                    msg: "Disconnect failed!".into(),
                                    urgency: Urgency::Critical,
                                    duration: None,
                                })));
                            }
                        }
                        _ = action_tx.send(Action::Refresh);
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
                Action::ShowToast(box_data) => {
                    let toast_req = *box_data;
                    app.toasts.push(Toast::new(
                        &app.config.ui.toast,
                        toast_req.title,
                        toast_req.msg,
                        toast_req.urgency,
                        toast_req.duration,
                    ));
                }
            }
        }
        Ok(())
    }
}
