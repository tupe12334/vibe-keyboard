//! Finds open Claude Code sessions, their titles and status, and focuses their terminal tab.
//!
//! This file holds the pure logic. Process spawns live in `claude_sessions/os.rs`.

mod focus_check;
mod new_session;
mod os;
mod process_list;
mod routine_filter;
mod session_list;
mod session_status;
mod terminal_scripts;
mod titles;
mod tmux_binary;
mod vscode_owner;

pub use focus_check::{focused_bundle, is_frontmost, ITERM_BUNDLE, TERMINAL_BUNDLE};
pub use new_session::new_session_script;
pub use os::{focus_session, list_sessions, open_new_session, send_prompt};
pub use process_list::{claude_processes, cwds_from_lsof};
pub use routine_filter::drop_routines;
pub use session_list::build_sessions;
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

/// True when `pid` is still a `claude` process on `tty`, per a fresh `ps` output. A tty that
/// closed and was reused by another tab fails this check.
pub fn is_session_on_tty(ps: &str, pid: u32, tty: &str) -> bool {
    claude_processes(ps)
        .iter()
        .any(|(live_pid, live_tty)| *live_pid == pid && live_tty == tty)
}

/// True for `ttysNNN` only. The tty goes into an `AppleScript` string, so anything else is refused.
pub fn is_valid_tty(tty: &str) -> bool {
    tty.strip_prefix("ttys")
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// `AppleScript` expression for `text`. Escapes backslash and double quote. `\n`, `\r` and `\t`
/// become `linefeed`, `return` and `tab` joined by `&`. Any other control character gives None.
pub fn applescript_string(text: &str) -> Option<String> {
    let mut out = String::from("\"");
    let mut joined = false;
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\" & linefeed & \""),
            '\r' => out.push_str("\" & return & \""),
            '\t' => out.push_str("\" & tab & \""),
            c if c.is_control() => return None,
            c => out.push(c),
        }
        joined |= matches!(c, '\n' | '\r' | '\t');
    }
    out.push('"');
    Some(if joined { format!("({out})") } else { out })
}

#[cfg(test)]
#[path = "claude_sessions_tests.rs"]
mod tests;
