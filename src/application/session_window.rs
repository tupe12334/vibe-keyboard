//! The session keys that act on the terminal window: BLINK and FULL SCREEN. Each runs in a thread,
//! off the device loop, because osascript can be slow.

use tracing::info;

use crate::domain::actions::ClaudeSession;
use crate::infrastructure::claude_sessions::{blink_session, full_screen_session};

/// BLINK: focus the session, then show the iTerm2 highlight on its pane.
pub(super) fn blink(session: ClaudeSession) {
    info!("claude sessions: blink {}", session.tty);
    std::thread::spawn(move || blink_session(&session.tty, session.app));
}

/// FULL SCREEN: focus the session, then toggle full screen on its window.
pub(super) fn full_screen(session: ClaudeSession) {
    info!("claude sessions: full screen {}", session.tty);
    std::thread::spawn(move || full_screen_session(&session.tty, session.app));
}
