use std::rc::Rc;

use nmrs::Network;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Cell, Paragraph, Row},
};

use crate::{
    app::App,
    tui::Tabs,
    ui::{input::Input, table::TableData},
};

pub mod input;
pub mod list;
pub mod table;

// Helper to calculate centered rect
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_password_popup(f: &mut Frame, input: &Input, selected_network: &Option<Network>) {
    // Dim the background.
    let dimming_block = ratatui::widgets::Block::default().bg(Color::Black);
    f.render_widget(dimming_block, f.area());

    let input_area = centered_rect(30, 6, f.area()); // Height 3 for one line + borders
    f.render_widget(ratatui::widgets::Clear, input_area); // Clear the table behind it

    // Hide password with asterisks for security
    let display_pass: String = input.value.chars().map(|_| '*').collect();

    let block = Block::bordered()
        .title(format!(
            " Password for {} ",
            selected_network
                .as_ref()
                .map(|n| n.ssid.as_str())
                .unwrap_or("Network")
        ))
        .border_style(Style::default().fg(Color::Yellow));

    let text = Paragraph::new(display_pass).block(block);
    f.render_widget(text, input_area);
    f.set_cursor_position(Position::new(
        // Draw the cursor at the current position in the input field.
        // This position can be controlled via the left and right arrow key
        input_area.x + input.cx as u16 + 1,
        // Move one line down, from the border to the input line
        input_area.y + 1,
    ))
}

pub fn render_all_tables(
    f: &mut Frame<'_>,
    body_chunks: Rc<[Rect]>,
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
    table::draw(
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
    table::draw(
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
    table::draw(
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
