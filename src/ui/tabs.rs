use crate::{
    app::{App, Focus},
    ui::{
        area::{self, Border},
        list::StatefulList,
        margin::Margin,
        style_config::StyleConfig,
    },
};
use nmrs::{ConnectType, Network, WifiDevice};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Row, Table, TableState},
};
use serde::{Deserialize, Serialize};

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

    pub fn draw(&self, f: &mut Frame<'_>, area: Rect, app: &mut App) {
        let active = matches!(app.focus, Focus::Tab(tab) if tab == *self);

        match self {
            Self::KnownNetworks | Self::AvailableNetworks => {
                let (config, list): (&TableConfig<NetworkColumnKind>, &mut StatefulList<Network>) =
                    match self {
                        Self::KnownNetworks => {
                            (&app.config.ui.known_networks, &mut app.known_networks)
                        }
                        Self::AvailableNetworks => (
                            &app.config.ui.available_networks,
                            &mut app.available_networks,
                        ),
                        Self::Devices => unreachable!(),
                    };

                let rows: Vec<Row> = list
                    .items
                    .iter()
                    .map(|net| -> Row<'_> {
                        let security = match net.security_features.preferred_connect_type() {
                            ConnectType::Open => "Open",
                            ConnectType::Psk => "WPA/WPA2",
                            ConnectType::Sae => "WPA3",
                            ConnectType::Eap => "Enterprise",
                            ConnectType::Owe => "OWE",
                            _ => "Unknown",
                        };

                        let (strength, bars) = match net.strength {
                            Some(strength) => {
                                let bars = match strength {
                                    81.. => "󰤨",
                                    61..=80 => "󰤥",
                                    41..=60 => "󰤢",
                                    21..=40 => "󰤟",
                                    _ => "󰤯",
                                };
                                (strength.to_string(), bars)
                            }
                            None => ("Unknown".to_string(), "󰤮"),
                        };

                        let connected = app
                            .network_manager
                            .current_network
                            .as_ref()
                            .is_some_and(|current| current.ssid == net.ssid);

                        let cells: Vec<Line> = config
                            .header
                            .columns
                            .iter()
                            .map(|c| match c.value {
                                NetworkColumnKind::Name => Line::from(net.ssid.as_str()),
                                NetworkColumnKind::Security => Line::from(security),
                                NetworkColumnKind::State => {
                                    Line::from(if connected { "Connected" } else { "-" })
                                }
                                NetworkColumnKind::Signal => {
                                    Line::from(format!("{}% {}", strength, bars))
                                }
                            })
                            .map(Line::centered)
                            .collect();

                        Row::new(cells)
                    })
                    .collect();

                draw(f, area, config, rows, &mut list.state, active);
            }

            Self::Devices => {
                let (config, list): (
                    &TableConfig<DeviceColumnKind>,
                    &mut StatefulList<WifiDevice>,
                ) = (&app.config.ui.devices, &mut app.devices);

                let security: &str = app
                    .network_manager
                    .current_network_info
                    .as_ref()
                    .map(|info| info.security.as_str())
                    .unwrap_or("-");

                let rows: Vec<Row> = list
                    .items
                    .iter()
                    .map(|dev| -> Row<'_> {
                        let cells: Vec<Line> = config
                            .header
                            .columns
                            .iter()
                            .map(|c| match c.value {
                                DeviceColumnKind::Name => Line::from(dev.interface.as_str()),
                                DeviceColumnKind::Powered => {
                                    Line::from(if dev.state.is_enabled() { "On" } else { "Off" })
                                }
                                DeviceColumnKind::State => Line::from(format!("{}", dev.state)),
                                DeviceColumnKind::Frequency => Line::from(format!(
                                    "{} MHz",
                                    dev.active_frequency_mhz.unwrap_or(0)
                                )),
                                DeviceColumnKind::Address => Line::from(dev.hw_address.as_str()),
                                DeviceColumnKind::Security => Line::from(security),
                            })
                            .map(Line::centered)
                            .collect();

                        Row::new(cells)
                    })
                    .collect();

                draw(f, area, config, rows, &mut list.state, active);
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TableConfig<K> {
    pub title: String,
    pub header: Header<K>,
    pub border: Border,
    #[serde(default)]
    pub margin: Margin,
}

#[derive(Deserialize, Serialize)]
pub struct Header<K> {
    pub columns: Vec<Column<K>>,
    #[serde(default)]
    pub style_normal: StyleConfig,
    #[serde(default)]
    pub style_active: StyleConfig,
}

#[derive(Deserialize, Serialize)]
pub struct Column<K> {
    pub title: String,
    pub value: K,
    pub constraint: Constraint,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum NetworkColumnKind {
    Name,
    Security,
    State,
    Signal,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum DeviceColumnKind {
    Name,
    Powered,
    State,
    Frequency,
    Address,
    Security,
}

pub fn draw<K>(
    f: &mut Frame,
    area: Rect,
    config: &TableConfig<K>,
    rows: Vec<Row<'_>>,
    state: &mut TableState,
    is_active: bool,
) {
    let area = area::fill_rect(area, config.margin);

    let (border_style, border_type, header_style, row_style) = if is_active {
        (
            config.border.style_active.format().unwrap_or_default(),
            config.border.type_active,
            config.header.style_active.format().unwrap_or_default(),
            Style::new().on_dark_gray(),
        )
    } else {
        (
            config.border.style_normal.format().unwrap_or_default(),
            config.border.type_normal,
            config.header.style_normal.format().unwrap_or_default(),
            Style::new(),
        )
    };

    let (columns, constraints): (Vec<Line>, Vec<Constraint>) = config
        .header
        .columns
        .iter()
        .map(|c| (Line::from(c.title.clone()).centered(), c.constraint))
        .unzip();
    let table = Table::new(rows, constraints)
        .header(Row::new(columns).style(header_style).bottom_margin(1))
        .block(
            Block::bordered()
                .title(config.title.clone())
                .border_style(border_style)
                .border_type(border_type),
        )
        .flex(Flex::Center)
        .row_highlight_style(row_style);

    f.render_stateful_widget(table, area, state);
}
