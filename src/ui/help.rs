use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::tui::{Focus, Tabs};

pub fn draw(f: &mut Frame, area: Rect, focus: &Focus) {
    let help_msg = match focus {
        Focus::Tab(tab) => match tab {
            Tabs::KnownNetworks => 
                "k,  Up | j,  Down | 󱁐  or ↵  Dis/connect | a Show All | d Remove | t Autoconnect | s Scan | p Share | ctrl+r Switch Mode | ⇄ Nav",
            
            Tabs::AvailableNetworks => 
                "k,  Up | j,  Down | 󱁐  or ↵  Connect | n Connect Hidden | a Show All | s Scan | ctrl+r Switch Mode | ⇄ Nav",
            
            Tabs::Devices => "s Scan | i Infos | o Toggle Power | ctrl+r Switch Mode | ⇄ Nav",
        },
        Focus::Popup(_) => "dsa",
    };

    f.render_widget(
        Paragraph::new(help_msg).alignment(Alignment::Center).style(Style::new().blue()),
        area,
    );
}
