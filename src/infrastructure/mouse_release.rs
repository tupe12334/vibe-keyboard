//! Clears a left mouse button the OS left held after the deck connects.
//!
//! The OS calls (button state, posting the release) live in `mouse_release/os.rs`.
//! This file holds the timing, the decision and the log line.

mod os;

use std::time::Duration;

pub use os::clear_stuck_left_button;

/// Wait this long after connect, so the OS settles before the button is read.
pub const CLEAR_DELAY: Duration = Duration::from_millis(300);

/// Post a release only when the left button reads as held.
pub const fn should_clear(left_down: bool) -> bool {
    left_down
}

/// The log line for one release attempt. `Ok` means the mouse-up was posted.
pub fn clear_log_line(result: Result<(), &str>) -> String {
    match result {
        Ok(()) => "cleared stuck left mouse button after deck connect".to_owned(),
        Err(reason) => format!("could not clear stuck left mouse button: {reason}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{clear_log_line, should_clear};

    #[test]
    fn clears_only_when_left_button_is_held() {
        assert!(should_clear(true));
        assert!(!should_clear(false));
    }

    #[test]
    fn logs_success_only_when_the_release_was_posted() {
        assert_eq!(
            clear_log_line(Ok(())),
            "cleared stuck left mouse button after deck connect"
        );
    }

    #[test]
    fn logs_the_reason_when_the_release_failed() {
        assert_eq!(
            clear_log_line(Err("could not create the mouse-up event")),
            "could not clear stuck left mouse button: could not create the mouse-up event"
        );
    }
}
