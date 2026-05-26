use anyhow::{Ok, Result};
use nmrs::{ConnectionError, Network, SavedConnection, WifiDevice, WifiSecurity};
use tokio::sync::mpsc::UnboundedSender;

use crate::{app::AppEvent, ui::toast::Urgency};

#[derive(Clone)]
pub struct NetworkManager {
    pub nmrs: nmrs::NetworkManager,
    pub event_sender: UnboundedSender<AppEvent>,
    pub current_connection: Option<Network>,
    pub devices: Vec<WifiDevice>,
    pub enabled: bool,
}

impl NetworkManager {
    pub async fn new(event_sender: UnboundedSender<AppEvent>) -> anyhow::Result<Self> {
        let nmrs = nmrs::NetworkManager::new().await?;
        let current_connection = nmrs.current_network().await?;

        let devices = nmrs.list_wifi_devices().await?;

        let enabled = nmrs.wifi_state().await?.enabled;

        Ok(Self {
            nmrs,
            event_sender,
            current_connection,
            devices,
            enabled,
        })
    }

    pub async fn get_devices(&mut self) -> Result<Vec<WifiDevice>, ConnectionError> {
        self.nmrs.list_wifi_devices().await
    }

    pub async fn scan_networks(&mut self) -> Result<Vec<Network>> {
        self.nmrs.scan_networks(None).await?;
        self.current_connection = self.nmrs.current_network().await?;

        let mut networks = self.nmrs.list_networks(None).await?;
        networks.sort_by(|a, b| b.strength.cmp(&a.strength));
        Ok(networks)
    }

    pub async fn networks_list(&mut self) -> anyhow::Result<(Vec<Network>, Vec<Network>)> {
        let scan_list = self.scan_networks().await.unwrap_or_default();

        let mut known_final = Vec::new();
        let mut new_final = Vec::new();

        for net in &scan_list {
            if self.has_saved_connection(&net.ssid).await? {
                known_final.push(net.clone());
            } else {
                new_final.push(net.clone());
            }
        }
        Ok((known_final, new_final))
    }

    pub async fn saved_connections(&mut self) -> Result<Vec<SavedConnection>, ConnectionError> {
        self.nmrs.list_saved_connections().await
    }

    pub async fn forget(&mut self, ssid: &str) -> Result<(), ConnectionError> {
        self.nmrs.forget(ssid).await
    }

    pub async fn connect(
        &mut self,
        ssid: &str,
        interface: Option<&str>,
        credentials: WifiSecurity,
    ) {
        match self.nmrs.connect(ssid, interface, credentials).await {
            std::result::Result::Ok(_) => {
                let msg = "Connected!";
                info!("{}", msg);
                let _ = self.event_sender.send(AppEvent::Toast(
                    None,
                    msg.into(),
                    Urgency::Success,
                    None,
                ));
            }
            Err(ConnectionError::NotFound) => {
                let msg = "Network not visible — is it in range?";
                error!("{}", msg);
                let _ = self.event_sender.send(AppEvent::Toast(
                    None,
                    msg.into(),
                    Urgency::Critical,
                    None,
                ));
            }
            Err(ConnectionError::AuthFailed) => {
                let _ = self.forget(ssid).await;
                let msg = "Wrong password!";
                error!("{}", msg);
                let _ = self.event_sender.send(AppEvent::Toast(
                    None,
                    msg.into(),
                    Urgency::Critical,
                    None,
                ));
            }
            Err(ConnectionError::Timeout) => {
                let msg = "Connection timed out — try increasing the timeout";
                error!("{}", msg);
                let _ = self.event_sender.send(AppEvent::Toast(
                    None,
                    msg.into(),
                    Urgency::Critical,
                    None,
                ));
            }
            Err(ConnectionError::DhcpFailed) => {
                let msg = "Failed to get an IP address";
                error!("{}", msg);
                let _ = self.event_sender.send(AppEvent::Toast(
                    None,
                    msg.into(),
                    Urgency::Critical,
                    None,
                ));
            }
            Err(e) => {
                let _ = self.event_sender.send(AppEvent::Toast(
                    None,
                    "Connection failed - check logs".into(),
                    Urgency::Critical,
                    None,
                ));
                error!("Connection failed: {}", e);
            }
        }
    }

    pub async fn is_connected(&mut self, ssid: &str) -> Result<bool, ConnectionError> {
        self.nmrs.is_connected(ssid).await
    }

    pub fn is_connected_cached(&mut self, ssid: &String) -> bool {
        self.current_connection.is_some() && self.current_connection.clone().unwrap().ssid == *ssid
    }

    pub async fn has_saved_connection(&mut self, ssid: &str) -> Result<bool, ConnectionError> {
        self.nmrs.has_saved_connection(ssid).await
    }
}
