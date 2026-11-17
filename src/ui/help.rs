use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::app::{Focus, Popups, Tabs};

pub fn draw(f: &mut Frame, area: Rect, focus: Focus) {
    let help_msg = match focus {
        Focus::Tab(Tabs::KnownNetworks) => {
            "k,  Up | j,  Down | 󱁐  or ↵  Dis/connect | d Forget | t Autoconnect | r Scan | ⇄ Nav"
        }
        Focus::Tab(Tabs::AvailableNetworks) => {
            "k,  Up | j,  Down | 󱁐  or ↵  Connect | r Scan | ⇄ Nav"
        }
        Focus::Tab(Tabs::Devices) => "r Scan | o Toggle Power | a Toggle Airplane | ⇄ Nav",
        Focus::Popup(Popups::Password) => "↵  Apply | ⇄ Hide/Show password | 󱊷  Discard",
    };

    f.render_widget(
        Paragraph::new(help_msg)
            .alignment(Alignment::Center)
            .style(Style::new().blue()),
        area,
    );
}
