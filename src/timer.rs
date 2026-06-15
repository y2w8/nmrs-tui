use std::time::Duration;

use crate::action::Action;

pub struct Timer {
    pub duration: Duration,
    pub remaining: Duration,
    pub on_finish: Action,
    pub enabled: bool,
}

#[allow(dead_code)]
impl Timer {
    pub fn new(duration: Duration, on_finish: Action, enabled: bool) -> Self {
        Self {
            duration,
            remaining: duration,
            on_finish,
            enabled,
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.remaining = self.remaining.saturating_sub(delta);
    }

    pub fn is_finished(&self) -> bool {
        self.remaining.is_zero()
    }

    pub fn reset(&mut self) {
        self.remaining = self.duration;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.reset();
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}
