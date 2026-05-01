use std::rc::Rc;

use crate::{app::App, tui::Tabs, ui::list::StatefulList};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Cell, Row, Table},
};

pub struct TableData<'a, T> {
    pub title: &'a str,
    pub header_cols: Vec<&'a str>,
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
    let row_style = if is_active {
        Style::new().on_blue().black()
    } else {
        Style::new()
    };
    let table = Table::new(table_data.cells.clone(), table_data.constraint.clone())
        .header(Row::new(table_data.header_cols.clone()).bold())
        .block(
            Block::bordered()
                .title(table_data.title)
                .border_style(border_style),
        )
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

pub fn draw_available_network(
    f: &mut Frame<'_>,
    body_chunks: &Rc<[Rect]>,
    app: &mut App,
    active_tab: &Tabs,
) {
    let rows: Vec<Row> = app
        .available_networks
        .items
        .iter()
        .map(|net| -> Row<'_> {
            Row::new(vec![
                Cell::from(net.ssid.clone()),
                Cell::from(if net.secured { "Secured" } else { "Open" }),
                Cell::from(format!("{}%", net.strength.unwrap_or(0))),
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
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
            cells: rows,
            list: &mut app.available_networks,
        },
        *active_tab == Tabs::AvailableNetworks,
    );
}

pub fn draw_devices(f: &mut Frame<'_>, body_chunks: &Rc<[Rect]>, app: &mut App, active_tab: &Tabs) {
    let rows: Vec<Row> = app
        .devices
        .items
        .iter()
        .map(|net| -> Row<'_> {
            Row::new(vec![
                Cell::from(net.interface.clone()),
                Cell::from(if app.network_manager.enabled {
                    "On"
                } else {
                    "Off"
                }),
            ])
        })
        .collect();
    draw(
        f,
        body_chunks[2],
        &mut TableData {
            title: " Device ",
            header_cols: vec!["Name", "Powered", "Scanning", "Frequency", "Security"],
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
        *active_tab == Tabs::Devices,
    );
}
