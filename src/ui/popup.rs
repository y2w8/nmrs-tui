use ratatui::{
    Frame,
    layout::{self, Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{self, Block, Paragraph},
};

use crate::{
    tui::Selected,
    ui::{self, Gaps, Position, input::Input},
};

#[derive(Default)]
pub struct Options {
    /// width of the popup.
    pub width: u16,

    /// height of the popup.
    pub height: u16,

    /// position of the popup.
    pub position: Position,

    /// gaps of the popup.
    pub gaps: Gaps,
}

pub fn popup_area(f: &mut Frame, opt: Options) -> Rect {
    let area = ui::position_rect(f.area(), opt.width, opt.height, opt.position, opt.gaps);
    f.render_widget(widgets::Clear, area); // Clear the area behind it
    area
}

pub fn draw_auth(f: &mut Frame, input: &Input, selected: &Option<Selected>, hidden_password: bool) {
    let popup_area = popup_area(
        f,
        Options {
            width: 60,
            height: 7,
            position: Position::Center,
            ..Default::default()
        },
    );

    if let Some(Selected::Network(net)) = selected {
        let block = Block::bordered().border_style(Style::default().fg(Color::Yellow).bold());

        let title = Line::from(vec![
            Span::raw("Enter the password for "),
            Span::styled(&net.ssid, Style::new().bold()),
        ]).alignment(Alignment::Center);

        let title_widget = Paragraph::new(title).block(block);
        f.render_widget(title_widget, popup_area);

        let input_area = Rect::new(
            popup_area.x + 1,
            popup_area.y + 3,
            popup_area.width.saturating_sub(2),
            1,
        );

        let input_chunks = Layout::horizontal([
            Constraint::Min(0),    // Password input
            Constraint::Length(4), // eye icon
        ])
        .split(input_area);

        let (password, icon): (String, &'static str) = if hidden_password {
            (input.value.chars().map(|_| '*').collect(), " 󰈉  ")
        } else {
            (input.value.to_string(), " 󰈈   ")
        };

        let password_widget = Paragraph::new(password).style(Style::new().on_dark_gray());
        f.render_widget(password_widget, input_chunks[0]);

        let icon_widget = Paragraph::new(icon).style(Style::new().green());
        f.render_widget(icon_widget, input_chunks[1]);

        let cx = input_chunks[0].x + input.cx as u16;
        let cx_max = input_chunks[0].x + input_chunks[0].width.saturating_sub(1) as u16;
        f.set_cursor_position(layout::Position::new(
            cx.min(cx_max),
            input_chunks[0].y,
        ))
    }
}
