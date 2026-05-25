use ratatui::{Frame, layout::{Alignment, Rect}, style::Style, widgets::Paragraph};

use crate::tui::Tabs;

pub fn draw(f: &mut Frame, area: Rect, active_tab: &Tabs) {
    let text = match active_tab {
        Tabs::KnownNetworks => Paragraph::new("k,  Up | j,  Down | 󱁐  or ↵  Dis/connect | a Show All | d Remove | t Autoconnect | s Scan | p Share | ctrl+r Switch Mode | ⇄ Nav"),
        Tabs::AvailableNetworks => Paragraph::new("k,  Up | j,  Down | 󱁐  or ↵  Connect | n Connect Hidden | a Show All | s Scan | ctrl+r Switch Mode | ⇄ Nav"),
        Tabs::Devices => Paragraph::new("s Scan | i Infos | o Toggle Power | ctrl+r Switch Mode | ⇄ Nav"),
    };

    f.render_widget(text.alignment(Alignment::Center).style(Style::new().blue()), area);
}
