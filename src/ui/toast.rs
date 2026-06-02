use std::{borrow::Cow, time::Duration};

use ratatui::{
    Frame,
    layout::Alignment,
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Paragraph},
};

use crate::ui::{
    Gaps, Position,
    popup::{self, Options},
};

#[derive(Clone, Copy, PartialEq)]
pub enum Urgency {
    Normal,
    Success,
    Warning,
    Critical,
}

#[derive(Clone)]
pub struct Toast {
    /// Toast title.
    pub title: Cow<'static, str>,
    /// Toast message.
    pub msg: Cow<'static, str>,
    /// Toast Urgency.
    pub urgency: Urgency,
    /// Toast Duration.
    pub duration: Duration,
}

impl Toast {
    pub fn new<T>(title: Option<T>, msg: T, urgency: Urgency, duration: Option<Duration>) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        let default_duration = match urgency {
            Urgency::Normal | Urgency::Success => Duration::from_secs_f32(3.0),
            Urgency::Warning => Duration::from_secs_f32(6.0),
            Urgency::Critical => Duration::from_secs_f32(10.0),
        };

        let title_unwraped: Cow<'static, str> = match title {
            Some(t) => t.into(),
            None => match urgency {
                Urgency::Normal => "Info".into(),
                Urgency::Success => "Success".into(),
                Urgency::Warning => "Warning".into(),
                Urgency::Critical => "Critical".into(),
            },
        };

        Self {
            title: title_unwraped,
            msg: msg.into(),
            urgency,
            duration: duration.unwrap_or(default_duration),
        }
    }
}

pub fn draw(f: &mut Frame, toasts: &[Toast]) {
    for (index, toast) in toasts.iter().enumerate() {
        let area = popup::popup_area(
            f,
            Options {
                width: 3 + toast.msg.chars().count() as u16,
                height: 4,
                position: Position::RightTop,
                gaps: Gaps {
                    top: 1 + (index as u16 * 4),
                    right: 1,
                    ..Default::default()
                },
            },
        );

        let border_style = match toast.urgency {
            Urgency::Normal => Style::new().gray(),
            Urgency::Success => Style::new().green(),
            Urgency::Warning => Style::new().yellow(),
            Urgency::Critical => Style::new().red(),
        };

        let title_style = match toast.urgency {
            Urgency::Normal => Style::new().gray(),
            Urgency::Success => Style::new().green(),
            Urgency::Warning => Style::new().yellow(),
            Urgency::Critical => Style::new().red(),
        };

        let title = Line::raw(toast.title.as_ref())
            .style(title_style.bold())
            .alignment(Alignment::Center);
        let msg = Line::raw(toast.msg.as_ref());

        let block = Block::bordered()
            .border_style(border_style)
            .border_type(BorderType::Thick);
        f.render_widget(Paragraph::new(vec![title, msg]).block(block), area);
    }
}
