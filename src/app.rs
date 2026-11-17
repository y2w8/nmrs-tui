use std::time::Duration;

use crate::{
    action::{Action, ActionHandler},
    config::Config,
    network_manager::NetworkManager,
    timer::Timer,
    ui::{input::Input, list::StatefulList, toast::Toast},
};
use anyhow::Result;
use nmrs::{Network, WifiDevice};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tabs {
    KnownNetworks,
    AvailableNetworks,
    Devices,
}

impl Tabs {
    pub fn next(self) -> Self {
        match self {
            Self::KnownNetworks => Self::AvailableNetworks,
            Self::AvailableNetworks => Self::Devices,
            Self::Devices => Self::KnownNetworks,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::KnownNetworks => Self::Devices,
            Self::AvailableNetworks => Self::KnownNetworks,
            Self::Devices => Self::AvailableNetworks,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Popups {
    Password,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    // Scanning,
    Connecting(String),
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum Focus {
    Tab(Tabs),
    Popup(Popups),
}

#[derive(Clone)]
pub enum Selected {
    Network(Network),
    Device(()),
}

pub struct App {
    pub action: ActionHandler,
    pub config: Config,
    pub network_manager: NetworkManager,
    pub should_quit: bool,

    // Data
    pub status: Status,
    pub input: Input,
    pub focus: Focus,
    pub last_focus: Focus,
    pub toasts: Vec<Toast>,

    // Timers
    pub scan: Timer,

    pub known_networks: StatefulList<Network>,
    pub available_networks: StatefulList<Network>,
    pub devices: StatefulList<WifiDevice>,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let action = ActionHandler::new();
        let network_manager = NetworkManager::new().await?;

        let device_list = network_manager.get_devices().await?;
        let (known_networks_list, available_networks_list) =
            network_manager.networks_list().await?;

        Ok(Self {
            action,
            config,
            network_manager,
            should_quit: false,

            // Data
            status: Status::None,
            input: Input::new(),
            focus: Focus::Tab(Tabs::KnownNetworks),
            last_focus: Focus::Tab(Tabs::KnownNetworks),
            toasts: Vec::new(),

            // Timers
            scan: Timer::new(Duration::from_secs(3), Action::Refresh, true),

            known_networks: StatefulList::new(known_networks_list),
            available_networks: StatefulList::new(available_networks_list),
            devices: StatefulList::new(device_list),
        })
    }
    pub fn timers_mut(&mut self) -> Vec<&mut Timer> {
        vec![&mut self.scan]
    }

    pub fn selected(&self) -> Option<Selected> {
        let focus = match self.focus {
            // If focus is popup we fallback to last_focus so we can get selected
            Focus::Popup(_) => self.last_focus,
            f => f,
        };

        match focus {
            Focus::Tab(Tabs::KnownNetworks) => self
                .known_networks
                .selected()
                .cloned()
                .map(Selected::Network),
            Focus::Tab(Tabs::AvailableNetworks) => self
                .available_networks
                .selected()
                .cloned()
                .map(Selected::Network),
            Focus::Tab(Tabs::Devices) => self.devices.selected().map(|_| Selected::Device(())),
            Focus::Popup(_) => None,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
