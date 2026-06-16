use ratatui::layout::{Constraint, Direction};
use serde::{Deserialize, Serialize};

use crate::ui::{
    popup::PopupConfig,
    table::{DeviceColumnKind, NetworkColumnKind, TableConfig},
    toast::ToastConfig,
};

pub mod area;
pub mod help;
pub mod input;
pub mod list;
pub mod margin;
pub mod popup;
pub mod style_config;
pub mod table;
pub mod toast;

#[derive(Deserialize, Serialize)]
pub struct Ui {
    pub layout: LayoutConfig,

    // Tables
    pub known_networks: TableConfig<NetworkColumnKind>,
    pub available_networks: TableConfig<NetworkColumnKind>,
    pub devices: TableConfig<DeviceColumnKind>,

    // Popups
    pub password_popup: PopupConfig,
    pub toast: ToastConfig,
}

#[derive(Deserialize, Serialize)]
pub struct LayoutConfig {
    pub direction: Direction,
    pub panels: Vec<PanelConfig>,
}

#[derive(Deserialize, Serialize)]
pub struct PanelConfig {
    pub kind: PanelKind,
    pub constraint: Constraint,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum PanelKind {
    KnownNetworks,
    AvailableNetworks,
    Devices,
    Help,
}
