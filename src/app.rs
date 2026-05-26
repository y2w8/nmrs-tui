use std::{borrow::Cow, time::Duration};

use crate::{
    network_manager::NetworkManager,
    ui::{list::StatefulList, toast::Urgency},
};
use anyhow::Result;
use nmrs::{Network, WifiDevice};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub enum AppEvent {
    Toast(Option<Cow<'static, str>>, Cow<'static, str>, Urgency, Option<Duration>),
    Refresh,
}

pub struct App {
    pub should_quit: bool,
    pub network_manager: NetworkManager,

    pub known_networks: StatefulList<Network>,
    pub available_networks: StatefulList<Network>,
    pub devices: StatefulList<WifiDevice>,

    pub event_sender: UnboundedSender<AppEvent>,
    pub event_receiver: UnboundedReceiver<AppEvent>,
}

impl App {
    pub async fn new() -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel::<AppEvent>();
        let mut network_manager = NetworkManager::new(event_sender.clone()).await?;

        let device_list = network_manager.get_devices().await.unwrap_or_default();
        let (known_networks_list, available_networks_list) =
            network_manager.networks_list().await?;

        Ok(Self {
            should_quit: false,
            network_manager,

            known_networks: StatefulList::with_items(known_networks_list),
            available_networks: StatefulList::with_items(available_networks_list),
            devices: StatefulList::with_items(device_list),
            event_sender,
            event_receiver,
        })
    }

    pub async fn refresh_networks(&mut self) -> Result<()> {
        let (known_networks, available_networks) = self.network_manager.networks_list().await?;
        self.known_networks.items = known_networks;
        self.available_networks.items = available_networks;
        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
