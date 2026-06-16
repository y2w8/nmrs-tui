use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, Deserialize, Serialize)]
#[serde(default)]
pub struct Margin {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
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
