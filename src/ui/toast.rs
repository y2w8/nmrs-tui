use std::{borrow::Cow, time::Duration};

use ratatui::{
    Frame,
    layout::Alignment,
    text::Line,
    widgets::{Block, Paragraph},
};
use serde::{Deserialize, Serialize};

use crate::ui::{
    area::{Border, Position},
    margin::Margin,
    popup::{self, PopupConfig},
    style_config::StyleConfig,
};

#[derive(Clone)]
pub struct Toast {
    pub title: Cow<'static, str>,
    pub msg: Cow<'static, str>,
    pub urgency: Urgency,
    pub duration: Duration,
}

#[derive(Deserialize, Serialize)]
pub struct ToastConfig {
    normal: UrgencyConfig,
    success: UrgencyConfig,
    warning: UrgencyConfig,
    critical: UrgencyConfig,
    position: Position,
    margin: Margin,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Urgency {
    Normal,
    Success,
    #[allow(dead_code)]
    Warning,
    Critical,
}

#[derive(Deserialize, Serialize)]
pub struct UrgencyConfig {
    default_title: String,
    title_style: StyleConfig,
    border: Border,
    default_duration_secs: f32,
}

impl Toast {
    pub fn new<T>(
        config: &ToastConfig,
        title: Option<T>,
        msg: T,
        urgency: Urgency,
        duration: Option<f32>,
    ) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        let urgency_config = match urgency {
            Urgency::Normal => &config.normal,
            Urgency::Success => &config.success,
            Urgency::Warning => &config.warning,
            Urgency::Critical => &config.critical,
        };

        let title: Cow<'static, str> = match title {
            Some(t) => t.into(),
            None => urgency_config.default_title.clone().into(),
        };

        Self {
            title,
            msg: msg.into(),
            urgency,
            duration: Duration::from_secs_f32(
                duration.unwrap_or(urgency_config.default_duration_secs),
            ),
        }
    }
}

pub fn draw(f: &mut Frame, config: &ToastConfig, toasts: &[Toast]) {
    for (index, toast) in toasts.iter().enumerate() {
        let area = popup::popup_rect(
            f,
            &PopupConfig {
                width: 4 + toast.msg.chars().count() as u16, // 4 = borders(2) + padding(2)
                height: 4,                                   // 4 = borders(2) + title(1) + msg(1)
                position: config.position,
                margin: Margin {
                    top: config.margin.top + (index as u16 * 4),
                    bottom: config.margin.bottom,
                    left: config.margin.left,
                    right: config.margin.right,
                },
            },
        );

        let urgency_config = match toast.urgency {
            Urgency::Normal => &config.normal,
            Urgency::Success => &config.success,
            Urgency::Warning => &config.warning,
            Urgency::Critical => &config.critical,
        };

        let title = Line::raw(toast.title.as_ref())
            .style(urgency_config.title_style.format().unwrap_or_default())
            .alignment(Alignment::Center);
        let msg = Line::raw(toast.msg.as_ref()).alignment(Alignment::Center);

        let block = Block::bordered()
            .border_style(
                urgency_config
                    .border
                    .style_normal
                    .format()
                    .unwrap_or_default(),
            )
            .border_type(urgency_config.border.type_normal);
        f.render_widget(Paragraph::new(vec![title, msg]).block(block), area);
    }
}
