use crate::{
    network, ui::list::StatefulList
};
use anyhow::Result;
use nmrs::Network;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub should_quit: bool,
    pub network_manager: network::Manager,

    pub devices: StatefulList<nmrs::Device>,
    pub known_networks: StatefulList<Network>,
    pub new_networks: StatefulList<Network>,

    pub input_mode: InputMode,
    pub password_input: String,
}

impl App {
    pub async fn new(mut network_manager: network::Manager) -> Result<Self> {
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
            new_networks: StatefulList::with_items(available_networks_list),

            input_mode: InputMode::Normal,
            password_input: "".to_string(),
        })
    }

    pub async fn scan_networks(&mut self) -> anyhow::Result<()> {
        // let known_names = self.network_manager.get_saved_networks().await.unwrap_or_default();
        let scan_list = self.network_manager.get_wifi_scan().await.unwrap_or_default();

        let mut known_final = Vec::new();
        let mut new_final = Vec::new();

        for net in &scan_list {
            if self.network_manager.has_saved_connection(&net.ssid).await? {
                known_final.push(net.clone());
            } else {
                new_final.push(net.clone());
            }
        }
        self.known_networks.items = known_final;
        self.new_networks.items = new_final;
        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
