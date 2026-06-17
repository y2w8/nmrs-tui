use std::{cmp::Reverse, time::Duration};

use anyhow::Context;
use nmrs::{ConnectionError, Network, NetworkInfo, SavedConnection, WifiDevice, WifiSecurity};
use tokio::time::{self, timeout};

#[derive(Clone)]
pub struct NetworkManager {
    pub nmrs: nmrs::NetworkManager,
    pub current_network: Option<Network>,
    pub current_network_info: Option<NetworkInfo>,
}

// TODO: AirPlane mode
impl NetworkManager {
    pub async fn new() -> anyhow::Result<Self> {
        let nmrs = nmrs::NetworkManager::new().await?;
        let current_network = nmrs.current_network().await?;
        let current_network_info = if let Some(network) = &current_network {
            Some(nmrs.show_details(network).await?)
        } else {
            None
        };

        Ok(Self {
            nmrs,
            current_network,
            current_network_info,
        })
    }

    pub async fn get_devices(&self) -> Result<Vec<WifiDevice>, ConnectionError> {
        self.nmrs.list_wifi_devices().await
    }

    pub async fn current_network(&self) -> Option<Network> {
        self.nmrs.current_network().await.ok()?
    }

    pub async fn show_details(&self, network: &Network) -> Result<NetworkInfo, ConnectionError> {
        self.nmrs.show_details(network).await
    }

    pub async fn scan_networks(&self) -> anyhow::Result<Vec<Network>> {
        self.nmrs
            .scan_networks(None)
            .await
            .context("Failed to trigger WiFi scan via NetworkManager")?;

        let mut networks = self.nmrs.list_networks(None).await?;
        networks.sort_by_key(|b| Reverse(b.strength));
        Ok(networks)
    }

    pub async fn networks_list(&self) -> Result<(Vec<Network>, Vec<Network>), ConnectionError> {
        let scan_list = self.scan_networks().await.unwrap_or_default();

        let mut known_final = Vec::new();
        let mut new_final = Vec::new();

        for net in &scan_list {
            if net.known {
                known_final.push(net.clone());
            } else {
                new_final.push(net.clone());
            }
        }
        Ok((known_final, new_final))
    }

    pub async fn _saved_connections(&self) -> Result<Vec<SavedConnection>, ConnectionError> {
        self.nmrs.list_saved_connections().await
    }

    pub async fn forget(&self, ssid: &str) -> Result<(), ConnectionError> {
        self.nmrs.forget(ssid).await
    }

    pub async fn connect(
        &self,
        ssid: &str,
        interface: Option<String>,
        credentials: WifiSecurity,
    ) -> Result<Result<(), ConnectionError>, time::error::Elapsed> {
        timeout(
            Duration::from_secs(30),
            self.nmrs.connect(ssid, interface.as_deref(), credentials),
        )
        .await
    }

    pub async fn _is_connected(&self, ssid: &str) -> Result<bool, ConnectionError> {
        self.nmrs.is_connected(ssid).await
    }

    #[allow(dead_code)]
    pub async fn has_saved_connection(&self, ssid: &str) -> Result<bool, ConnectionError> {
        self.nmrs.has_saved_connection(ssid).await
    }

    pub async fn disconnect(&self) -> Result<(), ConnectionError> {
        self.nmrs.disconnect(None).await
    }
}
