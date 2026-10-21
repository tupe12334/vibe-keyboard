//! Finds open Claude Code sessions, their titles and status, and focuses their terminal tab.
//!
//! This file holds the pure logic. Process spawns live in `claude_sessions/os.rs`.

mod applescript;
mod focus_check;
mod highlight;
mod new_session;
mod os;
mod process_list;
mod remote_control;
mod routine_filter;
mod session_list;
mod session_status;
mod terminal_scripts;
mod titles;
mod vscode_owner;

pub use applescript::{applescript_string, is_valid_tty};
pub use focus_check::{
    focused_bundle, is_frontmost, is_selected, scan_result, session_moved, Scan, ITERM_BUNDLE,
    TERMINAL_BUNDLE,
};
pub use highlight::highlight_target;
pub use new_session::new_session_script;
pub use os::{
    blink_session, focus_session, iterm_moved_from, list_sessions, open_new_session, send_prompt,
};
pub use process_list::{claude_processes, cwds_from_lsof};
pub use remote_control::{mark_remote_control, remote_control_on};
pub use routine_filter::drop_routines;
pub use session_list::{build_sessions, tab_owners};
pub use session_status::{age_label, label_for, status_for};
pub use terminal_scripts::{focus_scripts, send_text_scripts};
pub use titles::{parse_tab_titles, parse_title, summary_from_jsonl};
pub use vscode_owner::is_vscode_session;

use std::time::Duration;

#[cfg(test)]
use crate::domain::actions::Glyph;
#[cfg(test)]
use crate::domain::actions::{SessionStatus, TerminalApp};
#[cfg(test)]
use std::collections::HashMap;

/// A tty that printed within this long is `Active`. Older is `Stale`. Claude Code redraws while it
/// works, so a session that is quiet for a minute is waiting or idle.
pub const ACTIVE_WITHIN: Duration = Duration::from_secs(60);

/// A spinner keeps a session `Active` only this long after its tty last printed. Past this, a tab
/// left with a spinner (Claude exited mid-task) reads as `Stale`.
pub const SPINNER_ACTIVE_WITHIN: Duration = Duration::from_mins(10);

const CLAUDE_SUFFIX: &str = " (claude)";
/// Longest session name shown on a key. The key font fits about three short lines.
const SUMMARY_CHARS: usize = 60;

/// The pid to act on for `tty`, per a fresh `ps` output. `pid` while it still runs `claude` there.
/// Else the `claude` now on that tty, when the session restarted. Else None.
pub fn resolve_session_pid(ps: &str, pid: u32, tty: &str) -> Option<u32> {
    let on_tty: Vec<u32> = claude_processes(ps)
        .into_iter()
        .filter(|(_, live_tty)| live_tty == tty)
        .map(|(live_pid, _)| live_pid)
        .collect();
    if on_tty.contains(&pid) {
        Some(pid)
    } else {
        on_tty.first().copied()
    }
}

#[cfg(test)]
#[path = "claude_sessions_tests.rs"]
mod tests;
