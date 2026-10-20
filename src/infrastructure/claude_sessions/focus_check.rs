//! Pure rules for verifying that a session terminal really came to the front.
//!
//! The `osascript` focus scripts report `found` even when macOS refuses the `activate` from a
//! background process. These rules compare the frontmost bundle id with the terminal that was
//! focused. The process calls live in `claude_sessions/os/terminal.rs`.

/// Bundle id of iTerm2.
pub const ITERM_BUNDLE: &str = "com.googlecode.iterm2";
/// Bundle id of Terminal.app.
pub const TERMINAL_BUNDLE: &str = "com.apple.Terminal";

/// Bundle id the focus script at `index` targets: `0` is iTerm2, `1` is Terminal.app.
pub const fn focused_bundle(index: usize) -> Option<&'static str> {
    match index {
        0 => Some(ITERM_BUNDLE),
        1 => Some(TERMINAL_BUNDLE),
        _ => None,
    }
}

/// True when `front` is the `expected` bundle id. An unknown frontmost app is never a match.
pub fn is_frontmost(front: Option<&str>, expected: &str) -> bool {
    front == Some(expected)
}

#[cfg(test)]
#[path = "focus_check_tests.rs"]
mod tests;
