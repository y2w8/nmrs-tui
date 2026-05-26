use ratatui::layout::Rect;

pub mod help;
pub mod input;
pub mod list;
pub mod popup;
pub mod table;
pub mod toast;

#[allow(dead_code)]
#[derive(Default)]
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
pub struct Gaps {
    top: u16,
    bottom: u16,
    left: u16,
    right: u16,
}

// calculate area position
pub fn position_rect(
    area: Rect,
    mut width: u16,
    mut height: u16,
    position: Position,
    gaps: Gaps,
) -> Rect {
    // Make sure that it does not pass the area width and height to prevent crash.
    width = width.min(area.width);
    height = height.min(area.height);

    let x = match position {
        Position::LeftTop | Position::LeftCenter | Position::LeftBottom => {
            0_u16.saturating_add(gaps.left).saturating_sub(gaps.right)
        }

        Position::Top | Position::Center | Position::Bottom => (area.width.saturating_sub(width)
            / 2)
        .saturating_add(gaps.left)
        .saturating_sub(gaps.right),

        Position::RightTop | Position::RightCenter | Position::RightBottom => area
            .width
            .saturating_sub(width)
            .saturating_add(gaps.left)
            .saturating_sub(gaps.right),
    };

    let y = match position {
        Position::LeftTop | Position::Top | Position::RightTop => {
            0_u16.saturating_add(gaps.top).saturating_sub(gaps.bottom)
        }

        Position::LeftCenter | Position::Center | Position::RightCenter => {
            (area.height.saturating_sub(height) / 2)
                .saturating_add(gaps.top)
                .saturating_sub(gaps.bottom)
        }

        Position::LeftBottom | Position::Bottom | Position::RightBottom => area
            .height
            .saturating_sub(height)
            .saturating_add(gaps.top)
            .saturating_sub(gaps.bottom),
    };

    Rect::new(x, y, width, height)
}
