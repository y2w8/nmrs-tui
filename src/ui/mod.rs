use ratatui::{layout::Rect, widgets::BorderType};
use serde::{Deserialize, Serialize};

use crate::ui::{style_config::StyleConfig, table::{DeviceColumnKind, NetworkColumnKind, TableConfig}};

pub mod help;
pub mod input;
pub mod list;
pub mod popup;
pub mod table;
pub mod toast;
pub mod style_config;

#[derive(Deserialize, Serialize)]
pub struct Ui {
    pub known_networks: TableConfig<NetworkColumnKind>,
    pub available_networks: TableConfig<NetworkColumnKind>,
    pub devices: TableConfig<DeviceColumnKind>,
}

#[derive(Deserialize, Serialize)]
pub struct Border {
    style_normal: StyleConfig,
    style_active: StyleConfig,
    type_normal: BorderType,
    type_active: BorderType,
}

#[allow(dead_code)]
#[derive(Default, Deserialize, Serialize)]
pub enum Position {
    #[default]
    LeftTop,
    Top,
    RightTop,

    LeftCenter,
    Center,
    RightCenter,

    LeftBottom,
    Bottom,
    RightBottom,
}

#[derive(Default)]
pub struct Margin {
    top: u16,
    bottom: u16,
    left: u16,
    right: u16,
}

#[allow(dead_code)]
impl Margin {
    pub fn new(value: u16) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    pub fn horizontal(mut self, value: u16) -> Self {
        self.left = value;
        self.right = value;
        self
    }

    pub fn vertical(mut self, value: u16) -> Self {
        self.top = value;
        self.bottom = value;
        self
    }

    pub fn top(mut self, value: u16) -> Self {
        self.top = value;
        self
    }

    pub fn bottom(mut self, value: u16) -> Self {
        self.bottom = value;
        self
    }

    pub fn left(mut self, value: u16) -> Self {
        self.left = value;
        self
    }

    pub fn right(mut self, value: u16) -> Self {
        self.right = value;
        self
    }
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
