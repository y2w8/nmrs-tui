use std::rc::Rc;

use crate::{
    app::{App, Focus, Tabs},
    ui::list::StatefulList,
};
use nmrs::DeviceState;
use ratatui::{
    Frame, layout::{Constraint, Flex, Rect}, style::Style, text::Line, widgets::{Block, BorderType, Row, Table}
};

pub struct TableData<'a, T> {
    pub title: &'static str,
    pub header_cols: Vec<Line<'a>>,
    pub constraint: Vec<Constraint>,
    pub cells: Vec<Row<'a>>,
    pub list: &'a mut StatefulList<T>,
}

pub fn draw<Any>(f: &mut Frame, area: Rect, table_data: &mut TableData<Any>, is_active: bool) {
    let border_style = if is_active {
        Style::new().bold().green()
    } else {
        Style::new()
    };

    let border_type = if is_active {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    let header_style = if is_active {
        Style::new().bold().yellow()
    } else {
        Style::new()
    };

    let row_style = if is_active {
        Style::new().on_dark_gray()
    } else {
        Style::new()
    };
    let table = Table::new(table_data.cells.clone(), table_data.constraint.clone())
        .header(Row::new(table_data.header_cols.clone()).style(header_style).bottom_margin(1))
        .block(
            Block::bordered()
                .title(table_data.title)
                .border_style(border_style)
                .border_type(border_type),
        )
        .flex(Flex::Center)
        .row_highlight_style(row_style);

    f.render_stateful_widget(table, area, &mut table_data.list.state);
}

// TODO: this
// pub fn draw_saved_connections(
//     f: &mut Frame<'_>,
//     body_chunks: &Rc<[Rect]>,
//     app: &mut App,
// ) {
//     let rows: Vec<Row> = app
//         .known_networks
//         .items
//         .iter()
//         .map(|net| -> Row<'_> {
//             Row::new(vec![
//                 Line::from(net.ssid.clone()).centered(),
//                 Line::from(if net.secured { "Secured" } else { "Open" }).centered(),
//                 Line::from(if app.network_manager.is_connected_cached(&net.ssid) {
//                     "Connected"
//                 } else {
//                     "-"
//                 }).centered(),
//                 Line::from(format!("{}%", net.strength.unwrap_or(0))).centered(),
//             ])
//         })
//         .collect();
//     draw(
//         f,
//         body_chunks[0],
//         &mut TableData {
//             title: " Known Networks ",
//             header_cols: vec![
//                 Line::from("Name").centered(),
//                 Line::from("Security").centered(),
//                 Line::from("State").centered(),
//                 Line::from("Signal").centered()
//             ],
//             constraint: vec![
//                 Constraint::Percentage(40),
//                 Constraint::Percentage(20),
//                 Constraint::Percentage(20),
//                 Constraint::Percentage(20),
//             ],
//             cells: rows,
//             list: &mut app.known_networks,
//         },
//     );
// }

pub fn draw_known_network(
    f: &mut Frame<'_>,
    body_chunks: &Rc<[Rect]>,
    app: &mut App,
) {
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
                Line::from(if app.network_manager.current_network.as_ref().is_some_and(|current| current.ssid == net.ssid) {
                    "Connected"
                } else {
                    "-"
                }).centered(),
                Line::from(format!("{}% {}", strength, bars)).centered(),
            ])
        })
        .collect();
    draw(
        f,
        body_chunks[0],
        &mut TableData {
            title: " Known Networks ",
            header_cols: vec![
                Line::from("Name").centered(),
                Line::from("Security").centered(),
                Line::from("State").centered(),
                Line::from("Signal").centered()
            ],
            constraint: vec![
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
            cells: rows,
            list: &mut app.known_networks,
        },
        active,
    );
}

pub fn draw_available_network(
    f: &mut Frame<'_>,
    body_chunks: &Rc<[Rect]>,
    app: &mut App,
) {
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
        body_chunks[1],
        &mut TableData {
            title: " Available Networks ",
            header_cols: vec![
                Line::from("Name").centered(),
                Line::from("Security").centered(),
                Line::from("Signal").centered()
            ],
            constraint: vec![
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
            cells: rows,
            list: &mut app.available_networks,
        },
        active,
    );
}

pub fn draw_devices(f: &mut Frame<'_>, body_chunks: &Rc<[Rect]>, app: &mut App) {
    let active = if let Focus::Tab(tab) = app.focus
        && tab == Tabs::Devices
    {
        true
    } else {
        false
    };
    let rows: Vec<Row> = app
        .devices
        .items
        .iter()
        .map(|dev| -> Row<'_> {
            let freq = if let Some(current_connection) = &app.network_manager.current_network {
                format!("{} MHz", current_connection.frequency.unwrap_or_default())
            } else {
                "-".to_string()
            };

            Row::new(vec![
                Line::from(dev.interface.clone()).centered(),
                Line::from(if matches!(dev.state, DeviceState::Disconnected | DeviceState::Activated | DeviceState::Prepare | DeviceState::Config | DeviceState::NeedAuth | DeviceState::IpConfig | DeviceState::IpCheck | DeviceState::Secondaries | DeviceState::Deactivating) {
                    "On"
                } else {
                    "Off"
                }).centered(),
                Line::from(format!("{}", dev.state)).centered(),
                Line::from(freq).centered(),
                Line::from(dev.hw_address.to_string()).centered(),
            ])
        })
        .collect();
    draw(
        f,
        body_chunks[2],
        &mut TableData {
            title: " Devices ",
            header_cols: vec![
                Line::from("Name").centered(),
                Line::from("Powered").centered(),
                Line::from("State").centered(),
                Line::from("Frequency").centered(),
                Line::from("Address").centered(),
                Line::from("Security").centered(),
            ],
            constraint: vec![
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
            cells: rows,
            list: &mut app.devices,
        },
        active,
    );
}
