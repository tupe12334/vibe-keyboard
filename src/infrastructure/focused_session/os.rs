//! Process spawns for the focus check. Excluded from the coverage gate; the logic lives in
//! `focused_session.rs`.

use std::time::Duration;

use super::{parse_selected_tty, selected_tty_script, set_focus, terminal_for_bundle};
use crate::domain::actions::ClaudeSession;
use crate::infrastructure::frontmost::frontmost_bundle_id;
use crate::infrastructure::process::probe;

/// Hard limit for the tab query. Terminal apps can be slow to answer.
const TAB_TIMEOUT: Duration = Duration::from_millis(1500);

/// Returns `sessions` with the focused one marked. Asks the terminal only when it is frontmost and
/// lists a session, with one `osascript` at most. Blocks, so call it from the refresh thread.
pub fn with_focus(mut sessions: Vec<ClaudeSession>) -> Vec<ClaudeSession> {
    let focus = frontmost_bundle_id()
        .and_then(|bundle| terminal_for_bundle(&bundle))
        .filter(|app| sessions.iter().any(|session| session.app == *app))
        .and_then(|app| {
            let out = probe("osascript", &["-e", selected_tty_script(app)?], TAB_TIMEOUT)?;
            Some((app, parse_selected_tty(&out)?))
        });
    set_focus(
        &mut sessions,
        focus.as_ref().map(|(app, tty)| (*app, tty.as_str())),
    );
    sessions
}
