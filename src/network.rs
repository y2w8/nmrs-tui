use anyhow::{Ok, Result, anyhow};
use nmrs::{ConnectionError, Device, Network, NetworkManager, SavedConnection, WifiSecurity};

#[derive()]
pub struct Manager {
    pub network_manager: NetworkManager,
    pub current_connection: Option<Network>,
    pub devices: Vec<Device>,
    pub enabled: bool,
}

impl Manager {
    pub async fn new() -> anyhow::Result<Self> {
        let network_manager = NetworkManager::new().await?;
        let current_connection = network_manager.current_network().await?;

        let devices = network_manager.list_devices().await?;

        let enabled = network_manager.wifi_state().await?.enabled;

        Ok(Self {
            network_manager,
            current_connection,
            devices,
            enabled,
        })
    }

    pub async fn get_devices(&mut self) -> Result<Vec<Device>> {
        let devices = self.network_manager.list_devices().await?;
        let wireless_devices: Vec<Device> = devices
            .into_iter()
            .filter(|dev| dev.is_wireless())
            .collect();
        if wireless_devices.is_empty() {
            anyhow::bail!("No wireless device found")
        } else {
            Ok(wireless_devices)
        }
    }

    pub async fn get_wifi_scan(&mut self) -> Result<Vec<Network>> {
        self.network_manager.scan_networks(None).await?;
        self.current_connection = self.network_manager.current_network().await?;

        let mut networks = self.network_manager.list_networks(None).await?;
        networks.sort_by(|a, b| b.strength.cmp(&a.strength));
        Ok(networks)
    }

    pub async fn get_saved_networks(&mut self) -> anyhow::Result<Vec<SavedConnection>> {
        let saved_networks = self.network_manager.list_saved_connections().await?;
        Ok(saved_networks)
    }

    pub async fn forget(&mut self, ssid: &str) -> anyhow::Result<()> {
        self.network_manager.forget(ssid).await?;
        Ok(())
    }

    pub async fn connect(
        &mut self,
        ssid: &str,
        interface: Option<&str>,
        credentials: WifiSecurity,
    ) -> Result<()> {
        match self
            .network_manager
            .connect(ssid, interface, credentials)
            .await
        {
            std::result::Result::Ok(_) => {
                println!("Connected!");
                Ok(())
            }
            Err(ConnectionError::NotFound) => {
                warn!("Network not visible — is it in range?");
                Ok(())
            }
            Err(ConnectionError::AuthFailed) => {
                warn!("Wrong password");
                Ok(())
            }
            Err(ConnectionError::Timeout) => {
                error!("Connection timed out — try increasing the timeout");
                Ok(())
            }
            Err(ConnectionError::DhcpFailed) => {
                eprintln!("Failed to get an IP address");
                Ok(())
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
                Ok(())
            }
        }
    }

    pub async fn is_connected(&mut self, ssid: &String) -> Result<bool> {
        Ok(self.network_manager.is_connected(ssid).await?)
    }

    pub fn is_connected_cached(&mut self, ssid: &String) -> bool {
        self.current_connection.is_some() && self.current_connection.clone().unwrap().ssid == *ssid
    }

    pub async fn has_saved_connection(&mut self, ssid: &str) -> Result<bool> {
        Ok(self.network_manager.has_saved_connection(ssid).await?)
    }
}
