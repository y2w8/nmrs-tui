use ratatui::{
    widgets::{TableState},
};

pub struct StatefulList<'a, T> {
    pub state: TableState,
    pub items: Vec<&'a T>,
}
impl<'a, T> StatefulList<'a, T> {
    pub fn with_items(items: Vec<&'a T>) -> Self {
        Self {
            state: TableState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return; // Don't do anything if list is empty
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            _ => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return; // Don't do anything if list is empty
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            _ => 0,
        };
        self.state.select(Some(i));
    }
}

