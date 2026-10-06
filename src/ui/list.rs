use ratatui::widgets::TableState;

pub struct StatefulList<T> {
    pub state: TableState,
    pub items: Vec<T>,
    raw_items: Vec<T>,
}
impl<T: Clone> StatefulList<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut state = TableState::default();
        state.select_first();
        Self {
            state,
            raw_items: items.clone(),
            items,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.raw_items = items.clone();
        self.items = items;
        // clamp selection to new length, or select first if nothing selected
        match self.state.selected() {
            Some(i) if i >= self.items.len() => self.state.select_first(),
            None => self.state.select_first(),
            _ => {}
        }
    }

    pub fn filter<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.items = self
            .raw_items
            .iter()
            .filter(|item| predicate(item))
            .cloned()
            .collect();
        self.clamp_selection();
    }

    pub fn reset_filter(&mut self) {
        self.items = self.raw_items.clone();
        self.clamp_selection();
    }

    fn clamp_selection(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
        } else {
            match self.state.selected() {
                Some(i) if i >= self.items.len() => self.state.select_first(),
                None => self.state.select_first(),
                _ => {}
            }
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
