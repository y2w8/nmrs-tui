use ratatui::layout::Rect;

pub mod input;
pub mod list;
pub mod popup;
pub mod table;

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

pub enum Urgency {
    Success,
    Warning,
    Critical,
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
    width: u16,
    height: u16,
    position: Position,
    gaps: Option<Gaps>,
) -> Rect {
    let gaps = gaps.unwrap_or_default();
    let x = match position {
        Position::LeftTop | Position::LeftCenter | Position::LeftBottom => 0,
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
        Position::LeftTop | Position::Top | Position::RightTop => 0,
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
