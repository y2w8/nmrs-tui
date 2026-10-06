use ratatui::{
    Frame,
    layout::{self, Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
};

use crate::{
    app::{App, Focus, Status},
    ui::{input::InputMode, table::network_info},
};

pub fn draw(f: &mut Frame<'_>, area: Rect, app: &mut App) {
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
    let status_span: Vec<Span> = if let Some(status) = &app.status {
        match status {
            Status::Connecting(ssid) => vec![
                Span::styled("Connecting to ", Style::new().fg(Color::Yellow)),
                Span::styled(format!("{}...", ssid), Style::new().fg(Color::Green)),
            ],

            Status::Searching(search_query) => {
                let prefix = "Searching: ";
                let prefix_len = prefix.chars().count() as u16;
                let total_len = prefix_len + search_query.chars().count() as u16;

                let start_x = chunks[1].x + (chunks[1].width.saturating_sub(total_len) / 2);
                let cx = start_x + prefix_len + app.input.cx as u16;

                // cx_max = position + width - 1 (so the cursor does go outside the input area)
                let cx_max = chunks[1].x + chunks[1].width.saturating_sub(1);
                if app.focus == Focus::Header && app.input.mode == InputMode::Editing {
                    f.set_cursor_position(layout::Position::new(cx.min(cx_max), chunks[1].y));
                }

                vec![
                    Span::styled(prefix, Style::new().fg(Color::Yellow)),
                    Span::styled(search_query, Style::new().fg(Color::Reset)),
                ]
            }
        }
    } else {
        vec![Span::raw("")]
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
