use crate::{
    network_manager::NetworkManager, ui::list::StatefulList
};
use anyhow::Result;
use nmrs::Network;

pub struct App {
    pub should_quit: bool,
    pub network_manager: NetworkManager,

    pub devices: StatefulList<nmrs::Device>,
    pub known_networks: StatefulList<Network>,
    pub available_networks: StatefulList<Network>,
}

impl App {
    pub async fn new(mut network_manager: NetworkManager) -> Result<Self> {
        let scaned_networks = network_manager.get_wifi_scan().await.unwrap_or_default();
        let device_list = network_manager.get_devices().await.unwrap_or_default();


        let mut known_networks_list = Vec::new();
        let mut available_networks_list = Vec::new();

        for network in &scaned_networks {
            if network_manager.has_saved_connection(&network.ssid).await? {
                known_networks_list.push(network.clone());
            } else {
                available_networks_list.push(network.clone());
            }
        }

        Ok(Self {
            should_quit: false,
            network_manager,

            devices: StatefulList::with_items(device_list),
            known_networks: StatefulList::with_items(known_networks_list),
            available_networks: StatefulList::with_items(available_networks_list),
        })
    }

    pub async fn refresh_networks(&mut self) -> Result<()> {
        let (known_networks, available_networks) = self.network_manager.scan_networks().await?;
        self.known_networks.items = known_networks;
        self.available_networks.items = available_networks;
        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
