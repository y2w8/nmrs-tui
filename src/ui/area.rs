use ratatui::{layout::Rect, widgets::BorderType};
use serde::{Deserialize, Serialize};

use crate::ui::{margin::Margin, style_config::StyleConfig};

#[derive(Default, Clone, Copy, Deserialize, Serialize)]
pub enum Position {
    LeftTop,
    Top,
    RightTop,

    LeftCenter,
    #[default]
    Center,
    RightCenter,

    LeftBottom,
    Bottom,
    RightBottom,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Border {
    pub style_normal: StyleConfig,
    pub style_active: StyleConfig,
    pub type_normal: BorderType,
    pub type_active: BorderType,
}

// calculate area position
pub fn anchor_rect(
    area: Rect,
    mut width: u16,
    mut height: u16,
    position: Position,
    margin: Margin,
) -> Rect {
    // Make sure that it does not pass the area width and height to prevent crash.
    width = width.min(area.width);
    height = height.min(area.height);

    let x = match position {
        Position::LeftTop | Position::LeftCenter | Position::LeftBottom => 0_u16
            .saturating_add(margin.left)
            .saturating_sub(margin.right),

        Position::Top | Position::Center | Position::Bottom => (area.width.saturating_sub(width)
            / 2)
        .saturating_add(margin.left)
        .saturating_sub(margin.right),

        Position::RightTop | Position::RightCenter | Position::RightBottom => area
            .width
            .saturating_sub(width)
            .saturating_add(margin.left)
            .saturating_sub(margin.right),
    };

    let y = match position {
        Position::LeftTop | Position::Top | Position::RightTop => 0_u16
            .saturating_add(margin.top)
            .saturating_sub(margin.bottom),

        Position::LeftCenter | Position::Center | Position::RightCenter => {
            (area.height.saturating_sub(height) / 2)
                .saturating_add(margin.top)
                .saturating_sub(margin.bottom)
        }

        Position::LeftBottom | Position::Bottom | Position::RightBottom => area
            .height
            .saturating_sub(height)
            .saturating_add(margin.top)
            .saturating_sub(margin.bottom),
    };

    Rect::new(x, y, width, height)
}

pub fn fill_rect(area: Rect, margin: Margin) -> Rect {
    Rect::new(
        area.x.saturating_add(margin.left),
        area.y.saturating_add(margin.top),
        area.width
            .saturating_sub(margin.left)
            .saturating_sub(margin.right),
        area.height
            .saturating_sub(margin.top)
            .saturating_sub(margin.bottom),
    )
}
