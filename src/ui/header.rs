use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
};

use crate::app::{App, Status};

pub fn draw(f: &mut Frame<'_>, area: Rect, app: &App) {
    let chunks = Layout::horizontal([
        Constraint::Fill(1),
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
        let (strength, bars) = match current.strength {
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
