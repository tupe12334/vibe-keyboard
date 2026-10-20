//! Keys that edit text inputs on the deck: the number pad and the filter search.

use std::sync::{Arc, Mutex};

use tracing::info;

use super::{lock, render_screen};
use crate::domain::navigation::NavigationStack;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// A number-pad key: append a digit, clear, or delete the last digit.
pub(super) enum NumberEdit {
    Digit(char),
    Clear,
    Backspace,
}

/// Applies one number-pad edit and redraws the page.
pub(super) fn edit_number(
    edit: NumberEdit,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    match edit {
        NumberEdit::Digit(digit) => nav.input_number_append(digit),
        NumberEdit::Clear => nav.input_number_clear(),
        NumberEdit::Backspace => nav.input_number_backspace(),
    }
    let value = nav.input_number_value().unwrap_or("");
    info!("input number: value = {value}");
    render_screen(nav, handle, state, dev_state);
}

/// Puts the app in filter-search mode with an empty query.
pub(super) fn start_filter_input(state: &Arc<Mutex<tui::AppState>>) {
    info!("activating filter input mode");
    let mut s = lock(state);
    s.text_input_mode = true;
    s.text_input_value = String::new();
    s.text_input_confirmed = false;
}
