use std::rc::Rc;

use crate::{
    app::App,
    tui::{Focus, Tabs},
    ui::list::StatefulList,
};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Rect},
    style::Style,
    widgets::{Block, BorderType, Cell, Row, Table},
};

pub struct TableData<'a, T> {
    pub title: &'static str,
    pub header_cols: Vec<&'static str>,
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
pub fn draw_saved_connections(
    f: &mut Frame<'_>,
    body_chunks: &Rc<[Rect]>,
    app: &mut App,
    active_tab: &Tabs,
) {
    let rows: Vec<Row> = app
        .known_networks
        .items
        .iter()
        .map(|net| -> Row<'_> {
            Row::new(vec![
                Cell::from(net.ssid.clone()),
                Cell::from(if net.secured { "Secured" } else { "Open" }),
                Cell::from(if app.network_manager.is_connected_cached(&net.ssid) {
                    "Connected"
                } else {
                    "-"
                }),
                Cell::from(format!("{}%", net.strength.unwrap_or(0))),
            ])
        })
        .collect();
    draw(
        f,
        body_chunks[0],
        &mut TableData {
            title: " Known Networks ",
            header_cols: vec!["Name", "Security", "State", "Signal"],
            constraint: vec![
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
            cells: rows,
            list: &mut app.known_networks,
        },
        *active_tab == Tabs::KnownNetworks,
    );
}

pub fn draw_known_network(
    f: &mut Frame<'_>,
    body_chunks: &Rc<[Rect]>,
    app: &mut App,
    focus: &Focus,
) {
    let active = if let Focus::Tab(tab) = focus
        && *tab == Tabs::KnownNetworks
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
                Cell::from(net.ssid.clone()),
                Cell::from(security),
                Cell::from(if app.network_manager.is_connected_cached(&net.ssid) {
                    "Connected"
                } else {
                    "-"
                }),
                Cell::from(format!("{}% {}", strength, bars)),
            ])
        })
        .collect();
    draw(
        f,
        body_chunks[0],
        &mut TableData {
            title: " Known Networks ",
            header_cols: vec!["Name", "Security", "State", "Signal"],
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
    focus: &Focus,
) {
    let active = if let Focus::Tab(tab) = focus
        && *tab == Tabs::AvailableNetworks
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
                Cell::from(net.ssid.clone()),
                Cell::from(security),
                Cell::from(format!("{}% {}", strength, bars)),
            ])
        })
        .collect();
    draw(
        f,
        body_chunks[1],
        &mut TableData {
            title: " Available Networks ",
            header_cols: vec!["Name", "Security", "Signal"],
            constraint: vec![
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
            cells: rows,
            list: &mut app.available_networks,
        },
        active,
    );
}

pub fn draw_devices(f: &mut Frame<'_>, body_chunks: &Rc<[Rect]>, app: &mut App, focus: &Focus) {
    let active = if let Focus::Tab(tab) = focus
        && *tab == Tabs::Devices
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
            let freq = if let Some(current_connection) = &app.network_manager.current_connection {
                format!("{} MHz", current_connection.frequency.unwrap_or_default())
            } else {
                "-".to_string()
            };

            Row::new(vec![
                Cell::from(dev.interface.clone()),
                Cell::from(if app.network_manager.enabled {
                    "On"
                } else {
                    "Off"
                }),
                Cell::from(format!("{}", dev.state)),
                Cell::from(freq),
                Cell::from(dev.hw_address.to_string()),
            ])
        })
        .collect();
    draw(
        f,
        body_chunks[2],
        &mut TableData {
            title: " Device ",
            header_cols: vec![
                "Name",
                "Powered",
                "State",
                "Frequency",
                "Address",
                "Security",
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
