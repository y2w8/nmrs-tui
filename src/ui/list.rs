use ratatui::widgets::TableState;

pub struct StatefulList<T> {
    pub state: TableState,
    pub items: Vec<T>,
}
impl<T> StatefulList<T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut state = TableState::default();
        state.select_first();
        Self {
            state,
            items,
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        // clamp selection to new length, or select first if nothing selected
        match self.state.selected() {
            Some(i) if i >= self.items.len() => self.state.select_first(),
            None => self.state.select_first(),
            _ => {}
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
