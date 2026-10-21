//! The BLINK highlight: focus a session, then ask iTerm2 to flash its pane. Excluded from the
//! coverage gate; the target rules live in `highlight.rs`.

use std::fs::OpenOptions;
use std::io::Write;

use tracing::warn;

use super::terminal::focus_session;
use crate::domain::actions::TerminalApp;
use crate::infrastructure::claude_sessions::highlight_target;

/// Focus the session on `tty`, then ask iTerm2 for a fireworks highlight on that pane. Only iTerm2
/// reads the escape, so other apps get focus only. Blocks; call it from a thread.
pub fn blink_session(tty: &str, app: TerminalApp) {
    focus_session(tty);
    if app != TerminalApp::ITerm {
        return;
    }
    let Some((path, bytes)) = highlight_target(tty) else {
        return;
    };
    let written = OpenOptions::new()
        .write(true)
        .open(&path)
        .and_then(|mut device| device.write_all(bytes));
    if let Err(err) = written {
        warn!("claude sessions: highlight on {path} failed: {err}");
    }
}
