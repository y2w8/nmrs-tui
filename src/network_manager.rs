use anyhow::{Ok, Result};
use nmrs::{ConnectionError, Network, SavedConnection, WifiDevice, WifiSecurity};

#[derive()]
pub struct NetworkManager {
    pub nmrs: nmrs::NetworkManager,
    pub current_connection: Option<Network>,
    pub devices: Vec<WifiDevice>,
    pub enabled: bool,
}

impl NetworkManager {
    pub async fn new() -> anyhow::Result<Self> {
        let nmrs = nmrs::NetworkManager::new().await?;
        let current_connection = nmrs.current_network().await?;

        let devices = nmrs.list_wifi_devices().await?;

        let enabled = nmrs.wifi_state().await?.enabled;

        Ok(Self {
            nmrs,
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

    // TODO: add toast msg, send a signal or somthing like that.
    pub async fn connect(
        &mut self,
        ssid: &str,
        interface: Option<&str>,
        credentials: WifiSecurity,
    ) -> Result<()> {
        match self
            .nmrs
            .connect(ssid, interface, credentials)
            .await
        {
            std::result::Result::Ok(_) => {
                debug!("Connected!");
                Ok(())
            }
            Err(ConnectionError::NotFound) => {
                warn!("Network not visible — is it in range?");
                Ok(())
            }
            Err(ConnectionError::AuthFailed) => {
                self.forget(ssid).await?;
                warn!("Wrong password");
                Ok(())
            }
            Err(ConnectionError::Timeout) => {
                error!("Connection timed out — try increasing the timeout");
                Ok(())
            }
            Err(ConnectionError::DhcpFailed) => {
                error!("Failed to get an IP address");
                Ok(())
            }
            Err(e) => {
                error!("Connection failed: {}", e);
                Ok(())
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
