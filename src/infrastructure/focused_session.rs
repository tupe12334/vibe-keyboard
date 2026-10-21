//! Finds the Claude session whose terminal tab has focus right now.
//!
//! A session has focus when its terminal app is frontmost and that app's selected tab is the
//! session's tty. This file holds the pure logic. The process spawns live in `focused_session/os.rs`.

mod os;

pub use os::with_focus;

use crate::domain::actions::{ClaudeSession, TerminalApp};
use crate::infrastructure::claude_sessions::is_valid_tty;

/// The terminal app for a bundle id. None for every other app, so no tab query runs.
pub fn terminal_for_bundle(bundle: &str) -> Option<TerminalApp> {
    match bundle {
        "com.googlecode.iterm2" => Some(TerminalApp::ITerm),
        "com.apple.Terminal" => Some(TerminalApp::Terminal),
        _ => None,
    }
}

/// `AppleScript` that prints the device of the app's selected tab, e.g. `/dev/ttys003`.
pub const fn selected_tty_script(app: TerminalApp) -> Option<&'static str> {
    match app {
        TerminalApp::ITerm => Some(
            r#"tell application id "com.googlecode.iterm2" to tty of current session of current window"#,
        ),
        TerminalApp::Terminal => Some(
            r#"tell application id "com.apple.Terminal" to tty of selected tab of front window"#,
        ),
        TerminalApp::Other => None,
    }
}

/// The bare tty name from the script output. `/dev/ttys003\n` gives `ttys003`. None for anything
/// else, including an empty output from a failed script.
pub fn parse_selected_tty(output: &str) -> Option<String> {
    let tty = output.trim().strip_prefix("/dev/")?;
    is_valid_tty(tty).then(|| tty.to_string())
}

/// Sets `focused` on the one session that runs on `tty` in `app`. Every other session is unfocused.
pub fn set_focus(sessions: &mut [ClaudeSession], focus: Option<(TerminalApp, &str)>) {
    for session in sessions {
        session.focused = focus.is_some_and(|(app, tty)| session.app == app && session.tty == tty);
    }
}

#[cfg(test)]
#[path = "focused_session_tests.rs"]
mod tests;
