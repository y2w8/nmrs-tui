use nmrs::Network;
use ratatui::{
    Frame,
    layout::{self, Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{self, Block, BorderType, Paragraph},
};

use crate::ui::{self, Margin, Position, input::Input};

#[derive(Default)]
pub struct Options {
    pub width: u16,
    pub height: u16,
    pub position: Position,
    pub margin: Margin,
}

pub fn popup_rect(f: &mut Frame, opt: Options) -> Rect {
    let area = ui::anchor_rect(f.area(), opt.width, opt.height, opt.position, opt.margin);
    f.render_widget(widgets::Clear, area); // Clear the area behind it
    area
}

// TODO: padding.
pub fn draw_auth(f: &mut Frame, input: &Input, network: Network) {
    let popup_rect = popup_rect(
        f,
        Options {
            width: 60,
            height: 7,
            position: Position::Center,
            ..Default::default()
        },
    );

    let block = Block::bordered()
        .border_style(Style::default().green().bold())
        .border_type(BorderType::Thick);

    let title = Line::from(vec![
        Span::raw("Enter the password for "),
        Span::styled(&network.ssid, Style::new().bold()),
    ])
    .alignment(Alignment::Center);

    let title_widget = Paragraph::new(title).block(block);
    f.render_widget(title_widget, popup_rect);

    let input_area = Rect::new(
        popup_rect.x + 1,
        popup_rect.y + 3,
        popup_rect.width.saturating_sub(2),
        1,
    );

    let input_chunks = Layout::horizontal([
        Constraint::Min(0),    // Password input
        Constraint::Length(4), // eye icon
    ])
    .split(input_area);

    let (password, icon): (String, &'static str) = if input.hidden_password {
        (input.value.chars().map(|_| '*').collect(), " 󰈉  ")
    } else {
        (input.value.to_string(), " 󰈈   ")
    };

    let password_widget = Paragraph::new(password).style(Style::new().on_dark_gray());
    f.render_widget(password_widget, input_chunks[0]);

    let icon_widget = Paragraph::new(icon).style(Style::new().green());
    f.render_widget(icon_widget, input_chunks[1]);

    // Cursor position x
    let cx = input_chunks[0].x + input.cx as u16;

    // cx_max = position + width - 1 (so the cursor does overlap on eye icon)
    let cx_max = input_chunks[0].x + input_chunks[0].width.saturating_sub(1);
    f.set_cursor_position(layout::Position::new(cx.min(cx_max), input_chunks[0].y))
}
