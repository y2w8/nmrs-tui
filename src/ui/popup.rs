use nmrs::{Network, WifiSecurity};
use ratatui::{
    Frame,
    layout::{self, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph},
};

use crate::{
    tui::Selected,
    ui::{self, Gaps, Position, Urgency, input::Input},
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
    pub gaps: Option<Gaps>,

    /// dim the background.
    pub dim: Option<Color>,
}

pub fn popup_area(f: &mut Frame, opt: Options) -> Rect {
    if opt.dim.is_some() {
        // Dim the background.
        let dimming_block = ratatui::widgets::Block::default().bg(opt.dim.unwrap_or_default());
        f.render_widget(dimming_block, f.area());
    }

    let area = ui::position_rect(f.area(), opt.width, opt.height, opt.position, opt.gaps);
    f.render_widget(ratatui::widgets::Clear, area); // Clear the table behind it
    area
}

// TODO: draw toast
pub fn draw_toast(f: &mut Frame, msg: &str, urgency: Urgency) {
    let area = popup_area(
        f,
        Options {
            width: 30,
            height: 3,
            position: Position::RightBottom,
            gaps: Some(Gaps {
                bottom: 1,
                right: 1,
                ..Default::default()
            }),
            ..Default::default()
        },
    );

    let border_style = match urgency {
        Urgency::Success => Style::new().green(),
        Urgency::Warning => Style::new().yellow(),
        Urgency::Critical => Style::new().red(),
    };
    let block = Block::bordered().border_style(border_style);
    f.render_widget(Paragraph::new(msg).block(block), area);
}

pub fn draw_auth(f: &mut Frame, input: &Input, selected: &Option<Selected> ) {
    let input_area = popup_area(
        f,
        Options {
            width: 30,
            height: 3,
            position: Position::Center,
            dim: Some(Color::Black),
            ..Default::default()
        },
    );


    if let Some(Selected::Network(net)) = selected {
        let block = Block::bordered()
            .title(format!(" Password for {} ", net.ssid))
            .border_style(Style::default().fg(Color::Yellow));

        // Hide password with asterisks for security
        let display_pass: String = input.value.chars().map(|_| '*').collect();
        let text = Paragraph::new(display_pass).block(block);
        f.render_widget(text, input_area);
        f.set_cursor_position(layout::Position::new(
            // Draw the cursor at the current position in the input field.
            // Add 1 to position so it dont be on the border, min is forest the cursor can go mines
            // the borders
            (input_area.x + input.cx as u16 + 1).min(input_area.x + input_area.width.saturating_sub(2)),
            // Move one line down, from the border to the input line
            input_area.y + 1,
        ))
    }
}
