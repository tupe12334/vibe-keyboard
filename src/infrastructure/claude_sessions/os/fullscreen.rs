//! Runs the FULL SCREEN key: focus the session, then toggle full screen. Excluded from the coverage
//! gate; the steps and their order live in `claude_sessions/fullscreen.rs`.

use super::terminal::focus_session;
use super::FOCUS_TIMEOUT;
use crate::domain::actions::TerminalApp;
use crate::infrastructure::claude_sessions::{full_screen_steps, FullScreenStep};
use crate::infrastructure::process::run;

/// Focus the session on `tty`, then toggle full screen on its window. Runs the steps in order.
/// Blocks; call it from a thread.
pub fn full_screen_session(tty: &str, app: TerminalApp) {
    for step in full_screen_steps(app) {
        match step {
            FullScreenStep::Focus => {
                focus_session(tty);
            }
            FullScreenStep::Toggle(script) => {
                run("osascript", &["-e", &script], FOCUS_TIMEOUT);
            }
        }
    }
}
