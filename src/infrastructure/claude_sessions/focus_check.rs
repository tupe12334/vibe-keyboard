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

/// The `/dev/<tty>` path that iTerm2 and Terminal.app report for `tty`.
pub fn device_path(tty: &str) -> String {
    format!("/dev/{tty}")
}

/// True when the front session's tty is `tty`. An unknown front session is never a match.
pub fn is_selected(front: Option<&str>, tty: &str) -> bool {
    front.map(str::trim) == Some(device_path(tty).as_str())
}

/// True when iTerm2's current session is a tty other than `tty`. An unknown session is not moved.
pub fn session_moved(current: Option<&str>, tty: &str) -> bool {
    current
        .map(str::trim)
        .is_some_and(|c| !c.is_empty() && !is_selected(Some(c), tty))
}

/// One focus scan's result. `Failed` is an osascript error or no output. iTerm2 reorders its windows
/// during a scan, so a failed scan can be retried. `Missing` is a clean "no such tab".
#[derive(Debug, PartialEq, Eq)]
pub enum Scan {
    Found,
    Missing,
    Failed,
}

/// Reads one scan's stdout. Anything but `found` or `missing` counts as failed.
pub fn scan_result(out: Option<&str>) -> Scan {
    match out.map(str::trim) {
        Some("found") => Scan::Found,
        Some("missing") => Scan::Missing,
        _ => Scan::Failed,
    }
}

#[cfg(test)]
#[path = "focus_check_tests.rs"]
mod tests;
