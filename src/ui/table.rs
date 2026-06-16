use std::hash::Hash;

use crate::{
    app::{App, Focus, Tabs},
    ui::{self, Border, Margin, list::StatefulList, style_config::StyleConfig},
};
use ratatui::{
    Frame, layout::{Constraint, Flex, Rect}, style::Style, text::Line, widgets::{Block, BorderType, Row, Table}
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum NetworkColumnKind {
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

#[derive(Deserialize, Serialize)]
pub struct TableConfig<K> {
    pub title: String,
    pub header: Header<K>,
    pub border: Border,
}

#[derive(Deserialize, Serialize)]
pub struct Header<K> {
    pub columns: Vec<Column<K>>,
    pub style_normal: StyleConfig,
    pub style_active: StyleConfig,
}

#[derive(Deserialize, Serialize)]
pub struct Column<K> {
    pub title: String,
    pub value: K,
    pub constraint: Constraint,
}

pub struct Data<'a, T> {
    pub cells: Vec<Row<'a>>,
    pub list: &'a mut StatefulList<T>,
}

pub fn draw<Any, K: Eq + Hash>(
    f: &mut Frame,
    area: Rect,
    config: &TableConfig<K>,
    data: &mut Data<Any>,
    is_active: bool,
) {
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
    let table = Table::new(data.cells.clone(), constraints)
        .header(
            Row::new(columns)
                .style(header_style)
                .bottom_margin(1),
        )
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
    let active = if let Focus::Tab(tab) = app.focus
        && tab == Tabs::KnownNetworks
    {
        true
    } else {
        false
    };

    let rows: Vec<Row> = app
        .known_networks
        .items
        .iter()
        .map(|net| -> Row<'_> {
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

            Row::new(vec![
                Line::from(net.ssid.clone()).centered(),
                Line::from(security).centered(),
                Line::from(
                    if app
                        .network_manager
                        .current_network
                        .as_ref()
                        .is_some_and(|current| current.ssid == net.ssid)
                    {
                        "Connected"
                    } else {
                        "-"
                    },
                )
                .centered(),
                Line::from(format!("{}% {}", strength, bars)).centered(),
            ])
        })
        .collect();
    draw(
        f,
        ui::fill_rect(area, Margin::new(0).horizontal(1).top(1)),
        &app.config.ui.known_networks,
        &mut Data {
            cells: rows,
            list: &mut app.known_networks,
        },
        active,
    );
}

pub fn draw_available_network(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    let active = if let Focus::Tab(tab) = app.focus
        && tab == Tabs::AvailableNetworks
    {
        true
    } else {
        false
    };
    let rows: Vec<Row> = app
        .available_networks
        .items
        .iter()
        .map(|net| -> Row<'_> {
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

            Row::new(vec![
                Line::from(net.ssid.clone()).centered(),
                Line::from(security).centered(),
                Line::from(format!("{}% {}", strength, bars)).centered(),
            ])
        })
        .collect();
    draw(
        f,
        ui::fill_rect(area, Margin::new(0).horizontal(1)),
        &app.config.ui.available_networks,
        &mut Data {
            cells: rows,
            list: &mut app.available_networks,
        },
        active,
    );
}

pub fn draw_devices(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    let active = if let Focus::Tab(tab) = app.focus
        && tab == Tabs::Devices
    {
        true
    } else {
        false
    };

    let security = if let Some(network_info) = &app.network_manager.current_network_info {
        &network_info.security
    } else {
        &"-".to_string()
    };
    let rows: Vec<Row> = app
        .devices
        .items
        .iter()
        .map(|dev| -> Row<'_> {
            Row::new(vec![
                Line::from(dev.interface.clone()).centered(),
                Line::from(if dev.state.is_enabled() { "On" } else { "Off" }).centered(),
                Line::from(format!("{}", dev.state)).centered(),
                Line::from(format!("{} MHz", dev.active_frequency_mhz.unwrap_or(0))).centered(),
                Line::from(dev.hw_address.to_string()).centered(),
                Line::from(security.clone()).centered(),
            ])
        })
        .collect();
    draw(
        f,
        ui::fill_rect(area, Margin::new(0).horizontal(1)),
        &app.config.ui.devices,
        &mut Data {
            cells: rows,
            list: &mut app.devices,
        },
        active,
    );
}
