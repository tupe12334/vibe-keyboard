use super::Screen;

impl super::NavigationStack {
    /// Append a digit character to the current `InputNumber` value.
    /// No-op if not on the `InputNumber` screen.
    pub fn input_number_append(&mut self, digit: char) {
        if let Some(Screen::InputNumber { value }) = self.stack.last_mut() {
            value.push(digit);
        }
    }

    /// Clear the `InputNumber` value.
    /// No-op if not on the `InputNumber` screen.
    pub fn input_number_clear(&mut self) {
        if let Some(Screen::InputNumber { value }) = self.stack.last_mut() {
            value.clear();
        }
    }

    /// Delete the last character of the `InputNumber` value (backspace).
    /// No-op if not on the `InputNumber` screen.
    pub fn input_number_backspace(&mut self) {
        if let Some(Screen::InputNumber { value }) = self.stack.last_mut() {
            value.pop();
        }
    }

    /// Return the current `InputNumber` value, or `None` if not on that screen.
    pub fn input_number_value(&self) -> Option<&str> {
        if let Some(Screen::InputNumber { value }) = self.stack.last() {
            Some(value)
        } else {
            None
        }
    }
}
