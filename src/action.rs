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

#[derive(Clone)]
pub enum Actions {
    Quit,
    Tick(Duration),
    Refresh,
    SetFocus(Focus),
    SetInputMode(InputMode),

    // Network Commands
    NetworkScanResult(Vec<Network>, Vec<Network>, Vec<WifiDevice>),
    Connect(String, Option<String>, WifiSecurity),
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
        Option<Duration>,
    ),
}

pub struct Action {
    tx: UnboundedSender<Actions>,
    rx: UnboundedReceiver<Actions>,
}

impl Action {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Actions>();

        Self { tx, rx }
    }

    /// Returns action_tx.
    pub fn sender(&self) -> UnboundedSender<Actions> {
        self.tx.clone()
    }

    pub fn send(&self, action: Actions) {
        match self.tx.send(action) {
            Ok(_) => {}
            Err(e) => {
                error!("Action failed: {}", e)
            }
        }
    }

    pub fn drain(&mut self) -> Vec<Actions> {
        let mut actions: Vec<Actions> = Vec::new();
        while let Ok(action) = self.rx.try_recv() {
            actions.push(action);
        }
        actions
    }

    pub async fn handle_actions(app: &mut App) -> anyhow::Result<()> {
        for action in app.action.drain() {
            match action {
                Actions::Quit => app.quit(),
                Actions::Tick(delta) => {
                    // Toast duration timer
                    app.toasts.iter_mut().for_each(|toast| {
                        toast.duration = toast.duration.saturating_sub(delta);
                    });
                    app.toasts.retain(|toast| !toast.duration.is_zero());

                    // Timers
                    let finished: Vec<Actions> = app
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
                Actions::Refresh => {
                    if app.scan.enabled {
                        let action_tx = app.action.sender();
                        let network_manager = app.network_manager.clone();
                        tokio::spawn(async move {
                            match tokio::try_join!(
                                network_manager.networks_list(),
                                network_manager.get_devices()
                            ) {
                                Ok(((known, available), devices)) => {
                                    let _ = action_tx.send(Actions::NetworkScanResult(
                                        known, available, devices,
                                    ));
                                }
                                Err(e) => {
                                    error!("Refresh Failed: {}", e)
                                }
                            }
                        });
                    }
                }
                Actions::SetFocus(new_focus) => {
                    debug!("Set focus to: {:?}!", new_focus);
                    app.last_focus = app.focus;
                    app.focus = new_focus;
                }
                Actions::SetInputMode(new_inputmode) => {
                    debug!("Set InputMode to: {:?}!", new_inputmode);
                    app.input.mode = new_inputmode;
                    app.input.value.clear();
                    app.input.reset_cursor();
                }

                // Network
                Actions::NetworkScanResult(known, available, devices) => {
                    app.network_manager.current_network =
                        app.network_manager.current_network().await;

                    app.known_networks.set_items(known);
                    app.available_networks.set_items(available);
                    app.devices.set_items(devices);
                }
                Actions::Connect(ssid, interface, credentials) => {
                    let msg = format!("Connecting to {}...", ssid);
                    debug!("{}", msg);
                    app.action
                        .send(Actions::ShowToast(None, msg.into(), Urgency::Normal, None));

                    let action_tx = app.action.sender();
                    let network_manager = app.network_manager.clone();
                    tokio::spawn(async move {
                        match network_manager.connect(&ssid, interface, credentials).await {
                            Ok(Ok(_)) => {
                                let msg = "Connected!";
                                info!("{}", msg);
                                let _ = action_tx.send(Actions::ShowToast(
                                    None,
                                    msg.into(),
                                    Urgency::Success,
                                    None,
                                ));
                            }
                            Ok(Err(ConnectionError::NotFound)) => {
                                let msg = "Network not visible — is it in range?";
                                error!("{}", msg);
                                let _ = action_tx.send(Actions::ShowToast(
                                    None,
                                    msg.into(),
                                    Urgency::Critical,
                                    None,
                                ));
                            }
                            Ok(Err(ConnectionError::AuthFailed)) => {
                                let _ = action_tx.send(Actions::Forget(ssid.to_string()));
                                let msg = "Wrong password!";
                                error!("{}", msg);
                                let _ = action_tx.send(Actions::ShowToast(
                                    None,
                                    msg.into(),
                                    Urgency::Critical,
                                    None,
                                ));
                            }
                            Ok(Err(ConnectionError::Timeout)) => {
                                let msg = "Connection timed out — try increasing the timeout";
                                error!("{}", msg);
                                let _ = action_tx.send(Actions::ShowToast(
                                    None,
                                    msg.into(),
                                    Urgency::Critical,
                                    None,
                                ));
                            }
                            Ok(Err(ConnectionError::DhcpFailed)) => {
                                let msg = "Failed to get an IP address";
                                error!("{}", msg);
                                let _ = action_tx.send(Actions::ShowToast(
                                    None,
                                    msg.into(),
                                    Urgency::Critical,
                                    None,
                                ));
                            }
                            Ok(Err(e)) => {
                                let _ = action_tx.send(Actions::ShowToast(
                                    None,
                                    "Connection failed - check logs".into(),
                                    Urgency::Critical,
                                    None,
                                ));
                                error!("Connection failed: {}", e);
                            }
                            Err(_) => {
                                let msg = "Operation timed out!";
                                error!("{}", msg);
                                let _ = action_tx.send(Actions::ShowToast(
                                    None,
                                    msg.into(),
                                    Urgency::Critical,
                                    None,
                                ));
                            }
                        }
                    });
                }
                Actions::Forget(ssid) => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        if let Err(e) = network_manager.forget(&ssid).await {
                            error!("Forget failed: {}", e);
                        }
                        let _ = action_tx.send(Actions::Refresh);
                    });
                }
                Actions::TogglePower => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        let enabled = network_manager.nmrs.wifi_state().await.unwrap().enabled;
                        if let Err(e) = network_manager.nmrs.set_wireless_enabled(!enabled).await {
                            debug!("Toggle power failed: {}", e)
                        };

                        let msg = format!("Wifi {}", if !enabled { "On" } else { "Off" });

                        debug!("{}!", msg);
                        let _ = action_tx.send(Actions::ShowToast(
                            None,
                            msg.into(),
                            Urgency::Warning,
                            None,
                        ));
                        let _ = action_tx.send(Actions::Refresh);
                    });
                }
                Actions::Disconnect => {
                    let network_manager = app.network_manager.clone();
                    let action_tx = app.action.sender();
                    tokio::spawn(async move {
                        if let Err(e) = network_manager.disconnect().await {
                            error!("Disconnect failed: {}", e);
                        }
                        let _ = action_tx.send(Actions::Refresh);
                    });
                }

                // UI
                Actions::NextItem(tab) => match tab {
                    Tabs::KnownNetworks => app.known_networks.next(),
                    Tabs::AvailableNetworks => app.available_networks.next(),
                    Tabs::Devices => app.devices.next(),
                },
                Actions::PreviousItem(tab) => match tab {
                    Tabs::KnownNetworks => app.known_networks.previous(),
                    Tabs::AvailableNetworks => app.available_networks.previous(),
                    Tabs::Devices => app.devices.previous(),
                },
                Actions::ShowToast(title, msg, urgency, duration) => {
                    app.toasts.push(Toast::new(title, msg, urgency, duration));
                }
            }
        }
        Ok(())
    }
}
