use std::{hash::Hash, mem};

use crate::{
    app::{App, Focus, Tabs},
    ui::{
        area::{self, Border},
        list::StatefulList,
        margin::Margin,
        style_config::StyleConfig,
    },
};
use nmrs::Network;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Rect},
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Row, Table},
};
use serde::{Deserialize, Serialize};

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
pub enum KnownNetworkColumnKind {
    Name,
    Security,
    State,
    Signal,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum AvailableNetworkColumnKind {
    Name,
    Security,
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

pub struct Data<'a, T> {
    pub cells: Vec<Row<'a>>,
    pub list: &'a mut StatefulList<T>,
}

fn network_info<'a>(net: Network) -> (&'a str, String, &'a str) {
    let security = if net.is_psk {
        "Psk"
    } else if net.is_eap {
        "Enterprise"
    } else if net.secured {
        "Other"
    } else {
        "Open"
    };

    let (strength, bars) = if let Some(strength) = net.strength {
        let bars = if strength > 80 {
            "󰤨"
        } else if strength > 60 {
            "󰤥"
        } else if strength > 40 {
            "󰤢"
        } else if strength > 20 {
            "󰤟"
        } else {
            "󰤯"
        };
        (strength.to_string(), bars)
    } else {
        ("Unknown".to_string(), "󰤯")
    };

    (security, strength, bars)
}

pub fn draw<Any, K: Eq + Hash>(
    f: &mut Frame,
    area: Rect,
    config: &TableConfig<K>,
    data: &mut Data<Any>,
    is_active: bool,
) {
    let area = area::fill_rect(area, config.margin);

    let border_style: Style = if is_active {
        config.border.style_active.format().unwrap_or_default()
    } else {
        config.border.style_normal.format().unwrap_or_default()
    };

    let border_type: BorderType = if is_active {
        config.border.type_active
    } else {
        config.border.type_normal
    };

    let header_style: Style = if is_active {
        config.header.style_active.format().unwrap_or_default()
    } else {
        config.header.style_normal.format().unwrap_or_default()
    };

    let row_style = if is_active {
        Style::new().on_dark_gray()
    } else {
        Style::new()
    };

    let (columns, constraints): (Vec<Line>, Vec<Constraint>) = config
        .header
        .columns
        .iter()
        .map(|c| (Line::from(c.title.clone()).centered(), c.constraint))
        .unzip();
    let table = Table::new(mem::take(&mut data.cells), constraints)
        .header(Row::new(columns).style(header_style).bottom_margin(1))
        .block(
            Block::bordered()
                .title(config.title.clone())
                .border_style(border_style)
                .border_type(border_type),
        )
        .flex(Flex::Center)
        .row_highlight_style(row_style);

    f.render_stateful_widget(table, area, &mut data.list.state);
}

pub fn draw_known_network(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    let active = matches!(app.focus, Focus::Tab(Tabs::KnownNetworks));
    let columns = &app.config.ui.known_networks.header.columns;

    let rows: Vec<Row> = app
        .known_networks
        .items
        .iter()
        .map(|net| -> Row<'_> {
            let (security, strength, bars) = network_info(net.clone());

            let connected = app
                .network_manager
                .current_network
                .as_ref()
                .is_some_and(|current| current.ssid == net.ssid);

            let cells: Vec<Line> = columns
                .iter()
                .map(|c| match c.value {
                    KnownNetworkColumnKind::Name => Line::from(net.ssid.clone()),
                    KnownNetworkColumnKind::Security => Line::from(security),
                    KnownNetworkColumnKind::State => {
                        Line::from(if connected { "Connected" } else { "-" })
                    }
                    KnownNetworkColumnKind::Signal => Line::from(format!("{}% {}", strength, bars)),
                })
                .map(|line| line.centered())
                .collect();

            Row::new(cells)
        })
        .collect();
    draw(
        f,
        area,
        &app.config.ui.known_networks,
        &mut Data {
            cells: rows,
            list: &mut app.known_networks,
        },
        active,
    );
}

pub fn draw_available_network(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    let active = matches!(app.focus, Focus::Tab(Tabs::AvailableNetworks));
    let columns = &app.config.ui.available_networks.header.columns;

    let rows: Vec<Row> = app
        .available_networks
        .items
        .iter()
        .map(|net| -> Row<'_> {
            let (security, strength, bars) = network_info(net.clone());

            let cells: Vec<Line> = columns
                .iter()
                .map(|c| match c.value {
                    AvailableNetworkColumnKind::Name => Line::from(net.ssid.clone()),
                    AvailableNetworkColumnKind::Security => Line::from(security),
                    AvailableNetworkColumnKind::Signal => {
                        Line::from(format!("{}% {}", strength, bars))
                    }
                })
                .map(|line| line.centered())
                .collect();

            Row::new(cells)
        })
        .collect();
    draw(
        f,
        area,
        &app.config.ui.available_networks,
        &mut Data {
            cells: rows,
            list: &mut app.available_networks,
        },
        active,
    );
}

pub fn draw_devices(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    let active = matches!(app.focus, Focus::Tab(Tabs::Devices));
    let columns = &app.config.ui.devices.header.columns;

    let security: String = app
        .network_manager
        .current_network_info
        .as_ref()
        .map(|info| info.security.clone())
        .unwrap_or_else(|| "-".to_string());

    let rows: Vec<Row> = app
        .devices
        .items
        .iter()
        .map(|dev| -> Row<'_> {
            let cells: Vec<Line> = columns
                .iter()
                .map(|c| match c.value {
                    DeviceColumnKind::Name => Line::from(dev.interface.clone()),
                    DeviceColumnKind::Powered => {
                        Line::from(if dev.state.is_enabled() { "On" } else { "Off" })
                    }
                    DeviceColumnKind::State => Line::from(format!("{}", dev.state)),
                    DeviceColumnKind::Frequency => {
                        Line::from(format!("{} MHz", dev.active_frequency_mhz.unwrap_or(0)))
                    }
                    DeviceColumnKind::Address => Line::from(dev.hw_address.to_string()),
                    DeviceColumnKind::Security => Line::from(security.clone()),
                })
                .map(|line| line.centered())
                .collect();

            Row::new(cells)
        })
        .collect();
    draw(
        f,
        area,
        &app.config.ui.devices,
        &mut Data {
            cells: rows,
            list: &mut app.devices,
        },
        active,
    );
}
