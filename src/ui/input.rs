#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

pub struct Input {
    pub mode: InputMode,
    pub value: String,
    // cursor postion x.
    pub cx: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Normal,
            value: String::new(),
            cx: 0,
        }
    }
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cx.saturating_sub(1);
        self.cx = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cx.saturating_add(1);
        self.cx = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.value.insert(index, new_char);
        self.move_cursor_right();
    }

    // Returns the byte index based on the character position.
    // Since each character in a string can contain multiple bytes, it's necessary to calculate
    // the byte index based on the index of the character.
    fn byte_index(&mut self) -> usize {
        self.value
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cx)
            .unwrap_or(self.value.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cx != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cx;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.value.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.value.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.value = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.value.chars().count())
    }

    pub const fn reset_cursor(&mut self) {
        self.cx = 0;
    }
}
