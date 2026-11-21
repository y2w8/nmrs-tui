use std::{borrow::Cow, mem, time::Duration};

use nmrs::{ConnectionError, Network, SettingsPatch, WifiDevice, WifiSecurity};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    app::{App, Focus, Status},
    ui::{
        tabs::Tabs,
        toast::{Toast, Urgency},
    },
};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Tick(Duration),
    Refresh,
    SetFocus(Focus),
    SetStatus(Status),

    // Network Commands
    NetworkScanResult(Box<NetworkScanResult>),
    Connect(Box<ConnectRequest>),
    Forget { ssid: String },
    ToggleAutoconnect { ssid: String },
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
            match action {
                // App & Lifecycle
                Action::Quit => app.quit(),
                Action::Tick(delta) => Self::handle_tick(app, delta),
                Action::SetFocus(new_focus) => Self::set_focus(app, new_focus),
                Action::SetStatus(new_status) => {
                    debug!("Set status to: {:?}!", new_status);
                    app.status = new_status;
                }

                // Network Commands
                Action::Refresh => Self::spawn_refresh(app),
                Action::NetworkScanResult(box_data) => {
                    Self::apply_scan_result(app, *box_data).await;
                }
                Action::Connect(box_data) => Self::spawn_connect(app, *box_data),
                Action::Forget { ssid } => Self::spawn_forget(app, ssid),
                Action::ToggleAutoconnect { ssid } => Self::spawn_toggle_autoconnect(app, ssid),
                Action::ToggleAirplaneMode => Self::spawn_toggle_airplane(app),
                Action::TogglePower => Self::spawn_toggle_power(app),
                Action::Disconnect => Self::spawn_disconnect(app),

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

    // State Handlers

    fn handle_tick(app: &mut App, delta: Duration) {
        for toast in &mut app.toasts {
            toast.duration = toast.duration.saturating_sub(delta);
        }
        app.toasts.retain(|toast| !toast.duration.is_zero());

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

    fn set_focus(app: &mut App, new_focus: Focus) {
        debug!("Set focus to: {:?}!", new_focus);
        match new_focus {
            Focus::Popup(_) => {
                app.scan.disable();
                debug!("Scan disabled (popup is open)");
            }
            Focus::Tab(_) => app.scan.enable(),
        }
        app.last_focus = mem::replace(&mut app.focus, new_focus);
    }

    async fn apply_scan_result(app: &mut App, scan_result: NetworkScanResult) {
        app.network_manager.current_network = app.network_manager.current_network().await;
        app.network_manager.current_network_info =
            if let Some(network) = &app.network_manager.current_network {
                match app.network_manager.show_details(network).await {
                    Ok(conn_info) => Some(conn_info),
                    Err(_) => {
                        app.action
                            .send(Action::toast_err("Failed to get current network info!"));
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

    // Background Network Tasks

    fn spawn_refresh(app: &App) {
        if !app.scan.enabled {
            return;
        }
        let action_tx = app.action.sender();
        let network_manager = app.network_manager.clone();
        tokio::spawn(async move {
            match tokio::try_join!(
                network_manager.networks_list(),
                network_manager.get_devices()
            ) {
                Ok(((known, available), devices)) => {
                    _ = action_tx.send(Action::NetworkScanResult(Box::new(NetworkScanResult {
                        known,
                        available,
                        devices,
                    })));
                }
                Err(_) => {
                    _ = action_tx.send(Action::toast_err("Refresh failed!"));
                }
            }
        });
    }

    fn spawn_connect(app: &App, conn_req: ConnectRequest) {
        app.action
            .send(Action::SetStatus(Status::Connecting(conn_req.ssid.clone())));

        let action_tx = app.action.sender();
        let network_manager = app.network_manager.clone();
        tokio::spawn(async move {
            let toast = match network_manager
                .connect(&conn_req.ssid, conn_req.interface, conn_req.credentials)
                .await
            {
                Ok(Ok(_)) => Action::toast_ok("Connected!"),
                Ok(Err(ConnectionError::NotFound)) => {
                    Action::toast_err("Network not visible — is it in range?")
                }
                Ok(Err(ConnectionError::AuthFailed)) => {
                    _ = action_tx.send(Action::Forget {
                        ssid: conn_req.ssid,
                    });
                    Action::toast_err("Wrong password!")
                }
                Ok(Err(ConnectionError::Timeout)) => {
                    Action::toast_err("Connection timed out — try increasing the timeout")
                }
                Ok(Err(ConnectionError::DhcpFailed)) => {
                    Action::toast_err("Failed to get an IP address")
                }
                Ok(Err(_)) => Action::toast_err("Connection failed!"),
                Err(_) => {
                    let msg = "Operation timed out!";
                    error!("{}", msg);
                    Action::toast_err(msg)
                }
            };
            _ = action_tx.send(toast);
            _ = action_tx.send(Action::SetStatus(Status::None));
            _ = action_tx.send(Action::Refresh);
        });
    }

    fn spawn_forget(app: &App, ssid: String) {
        let network_manager = app.network_manager.clone();
        let action_tx = app.action.sender();
        tokio::spawn(async move {
            let toast = match network_manager.forget(&ssid).await {
                Ok(_) => Action::toast_ok("Forgotten!"),
                Err(_) => Action::toast_err("Forget failed!"),
            };
            _ = action_tx.send(toast);
            _ = action_tx.send(Action::Refresh);
        });
    }

    fn spawn_toggle_autoconnect(app: &App, ssid: String) {
        let network_manager = app.network_manager.clone();
        let action_tx = app.action.sender();
        tokio::spawn(async move {
            let uuid = match network_manager.get_saved_connection_uuid(&ssid).await {
                Ok(Some(uuid)) => uuid,
                Ok(None) => return debug!("can't find uuid for {}!", ssid),
                Err(_) => {
                    _ = action_tx.send(Action::toast_err("Failed to look up saved connection!"));
                    return;
                }
            };
            let saved_conn = match network_manager.get_saved_connection(&uuid).await {
                Ok(conn) => conn,
                Err(_) => {
                    _ = action_tx.send(Action::toast_err("Failed to get the saved connection!"));
                    return;
                }
            };
            let mut patch = SettingsPatch::default();
            patch.autoconnect = Some(!saved_conn.autoconnect);

            let toast = match network_manager.update_saved_connection(&uuid, patch).await {
                Ok(_) => Action::toast_ok(format!(
                    "Auto Connect: {}",
                    if !saved_conn.autoconnect { "On" } else { "Off" }
                )),
                Err(_) => Action::toast_err("Failed to update the saved connection!"),
            };
            _ = action_tx.send(toast);
        });
    }

    fn spawn_toggle_airplane(app: &App) {
        let network_manager = app.network_manager.clone();
        let action_tx = app.action.sender();
        tokio::spawn(async move {
            let enabled = match network_manager.airplane_mode_state().await {
                Ok(state) => state.is_airplane_mode(),
                Err(_) => {
                    _ = action_tx.send(Action::toast_err("Failed to get airplane mode state!"));
                    return;
                }
            };

            let toast = match network_manager.set_airplane_mode(!enabled).await {
                Ok(_) => {
                    debug!("Airplane set to {}", !enabled);
                    Action::toast_ok(format!(
                        "Airplane mode {}",
                        if !enabled { "On" } else { "Off" }
                    ))
                }
                Err(_) => Action::toast_err("Toggle airplane mode failed!"),
            };
            _ = action_tx.send(toast);
            _ = action_tx.send(Action::Refresh);
        });
    }

    fn spawn_toggle_power(app: &App) {
        let network_manager = app.network_manager.clone();
        let action_tx = app.action.sender();
        tokio::spawn(async move {
            let enabled = match network_manager.wifi_state().await {
                Ok(wifi_state) => wifi_state.enabled,
                Err(_) => {
                    _ = action_tx.send(Action::toast_err("Failed to get wifi state!"));
                    return;
                }
            };

            let toast = match network_manager.set_wireless_enabled(!enabled).await {
                Ok(_) => {
                    debug!("Wifi radio set to {}", !enabled);
                    Action::toast_ok(format!("Wifi {}", if !enabled { "On" } else { "Off" }))
                }
                Err(_) => Action::toast_err("Toggle power failed!"),
            };
            _ = action_tx.send(toast);
            _ = action_tx.send(Action::Refresh);
        });
    }

    fn spawn_disconnect(app: &App) {
        let network_manager = app.network_manager.clone();
        let action_tx = app.action.sender();
        tokio::spawn(async move {
            let toast = match network_manager.disconnect().await {
                Ok(_) => Action::toast_ok("Disconnected!"),
                Err(_) => Action::toast_err("Disconnect failed!"),
            };
            _ = action_tx.send(toast);
            _ = action_tx.send(Action::Refresh);
        });
    }
}

impl Action {
    pub fn toast(
        title: Option<Cow<'static, str>>,
        msg: impl Into<Cow<'static, str>>,
        urgency: Urgency,
        duration: Option<f32>,
    ) -> Self {
        Self::ShowToast(Box::new(ToastRequest {
            title,
            msg: msg.into(),
            urgency,
            duration,
        }))
    }

    pub fn toast_ok(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::toast(None, msg, Urgency::Success, None)
    }

    pub fn toast_err(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::toast(None, msg, Urgency::Critical, None)
    }
}
