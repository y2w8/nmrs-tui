use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
};

use crate::{
    app::{App, Status},
    ui::table::network_info,
};

pub fn draw(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    let chunks = Layout::horizontal([
        Constraint::Fill(12),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .split(area);

    let left_line = Line::from(vec![Span::styled(
        " nmrs-tui",
        Style::new().bold().fg(Color::Yellow),
    )])
    .alignment(Alignment::Left);

    //  Center: status
    let status_span = match &app.status {
        Status::Connecting(ssid) => Span::styled(
            format!("Connecting to {}...", ssid),
            Style::new().fg(Color::Green),
        ),
        Status::None => Span::raw(""),
    };
    let center_line = Line::from(status_span).alignment(Alignment::Center);

    //  Right: Currently connected network
    let right_line = if let Some(current) = &app.network_manager.current_network {
        let (_, strength, bars) = network_info(current);
        Line::from(vec![
            Span::styled(format!("{} ", bars), Style::new().fg(Color::Green)),
            Span::raw(format!("{} ({}%) ", current.ssid, strength)),
        ])
    } else {
        Line::from(vec![Span::styled(
            "󰤮 Disconnected ",
            Style::new().fg(Color::DarkGray),
        )])
    }
    .alignment(Alignment::Right);

    f.render_widget(left_line, chunks[0]);
    f.render_widget(center_line, chunks[1]);
    f.render_widget(right_line, chunks[2]);
}
