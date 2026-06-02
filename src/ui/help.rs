use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::app::{Focus, Popups, Tabs};

pub fn draw(f: &mut Frame, area: Rect, focus: Focus) {
    let help_msg = match focus {
        Focus::Tab(tab) => match tab {
            Tabs::KnownNetworks => 
                "k,  Up | j,  Down | 󱁐  or ↵  Dis/connect | a Show All | d Remove | t Autoconnect | r Scan | p Share | ⇄ Nav",
            
            Tabs::AvailableNetworks => 
                "k,  Up | j,  Down | 󱁐  or ↵  Connect | r Scan | ⇄ Nav",
            
            Tabs::Devices => "r Scan | i Infos | o Toggle Power | ⇄ Nav",
        },
        Focus::Popup(popup) => match popup {
            Popups::Password => "↵  Apply | ⇄ Hide/Show password | 󱊷  Discard",
        },
    };

    f.render_widget(
        Paragraph::new(help_msg).alignment(Alignment::Center).style(Style::new().blue()),
        area,
    );
}
