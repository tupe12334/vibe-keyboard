//! Clears a left mouse button the OS left held after the deck connects.
//!
//! The OS calls (button state, posting the release) live in `mouse_release/os.rs`.
//! This file holds the timing and the decision.

mod os;

use std::time::Duration;

pub use os::clear_stuck_left_button;

/// Wait this long after connect, so the OS settles before the button is read.
pub const CLEAR_DELAY: Duration = Duration::from_millis(300);

/// Post a release only when the left button reads as held.
pub const fn should_clear(left_down: bool) -> bool {
    left_down
}

#[cfg(test)]
mod tests {
    use super::should_clear;

    #[test]
    fn clears_only_when_left_button_is_held() {
        assert!(should_clear(true));
        assert!(!should_clear(false));
    }
}
