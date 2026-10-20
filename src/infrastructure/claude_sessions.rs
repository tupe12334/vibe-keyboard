//! Finds open Claude Code sessions, their titles and status, and focuses their terminal tab.
//!
//! This file holds the pure logic. Process spawns live in `claude_sessions/os.rs`.

mod new_session;
mod os;
mod terminal_scripts;
mod titles;

pub use new_session::new_session_script;
pub use os::{focus_session, list_sessions, open_new_session, send_prompt};
pub use terminal_scripts::{focus_scripts, send_text_scripts};
pub use titles::{parse_tab_titles, parse_title, summary_from_jsonl};

use std::collections::HashMap;
use std::time::Duration;

use crate::domain::actions::{ClaudeSession, Glyph, SessionStatus, TerminalApp};

/// A tty that printed within this long is `Active`. Older is `Stale`. Claude Code redraws while it
/// works, so a session that is quiet for a minute is waiting or idle.
pub const ACTIVE_WITHIN: Duration = Duration::from_secs(60);

/// A spinner keeps a session `Active` only this long after its tty last printed. Past this, a tab
/// left with a spinner (Claude exited mid-task) reads as `Stale`.
pub const SPINNER_ACTIVE_WITHIN: Duration = Duration::from_mins(10);

const CLAUDE_SUFFIX: &str = " (claude)";
/// Longest session name shown on a key. The key font fits about three short lines.
const SUMMARY_CHARS: usize = 60;

/// `(pid, tty)` for each `claude` process that has a terminal, from `ps -Ao pid=,tty=,comm=`.
/// Skips `??` (no terminal, for example a background helper).
pub fn claude_processes(ps: &str) -> Vec<(u32, String)> {
    ps.lines()
        .filter_map(|line| {
            let (pid, rest) = line.trim_start().split_once(' ')?;
            let (tty, comm) = rest.trim_start().split_once(' ')?;
            let name = comm.trim().rsplit('/').next()?;
            if name != "claude" || tty == "??" {
                return None;
            }
            Some((pid.parse().ok()?, tty.to_string()))
        })
        .collect()
}

/// Working directory per pid from `lsof -a -d cwd -Fpn -p <pids>`. A `p` line starts a process
/// and its `n` line holds the path.
pub fn cwds_from_lsof(output: &str) -> HashMap<u32, String> {
    let mut cwds = HashMap::new();
    let mut pid: Option<u32> = None;
    for line in output.lines() {
        if let Some(text) = line.strip_prefix('p') {
            pid = text.parse().ok();
        } else if let (Some(id), Some(path)) = (pid, line.strip_prefix('n')) {
            if !path.is_empty() {
                cwds.insert(id, path.to_string());
            }
        }
    }
    cwds
}

/// Short name for a working directory: `~` for home, else the last path component.
pub fn label_for(cwd: &str, home: &str) -> String {
    if cwd == home {
        return "~".into();
    }
    cwd.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("/")
        .to_string()
}

/// True for the spinner glyphs Claude Code cycles while it works: `◐◑◒◓` and braille.
const fn is_spinner(c: char) -> bool {
    matches!(c, '\u{25D0}'..='\u{25D3}' | '\u{2800}'..='\u{28FF}')
}

/// Activity status from the tty age and the title glyph. A spinner means Claude is mid-task, so it
/// is `Active` while the tty is at most `SPINNER_ACTIVE_WITHIN` old. Otherwise the age decides
/// against `ACTIVE_WITHIN`. `✳` does not change the result.
pub fn status_for(age: Duration, title_glyph: Option<Glyph>) -> SessionStatus {
    let spinning = title_glyph == Some(Glyph::Spinner) && age <= SPINNER_ACTIVE_WITHIN;
    if spinning || age <= ACTIVE_WITHIN {
        SessionStatus::Active
    } else {
        SessionStatus::Stale
    }
}

/// Short age for a key corner: `NOW` under a minute, then `2M`, `1H`, `3D`. Empty when unknown.
pub fn age_label(age: Option<Duration>) -> String {
    let Some(age) = age else {
        return String::new();
    };
    let secs = age.as_secs();
    match secs {
        0..=59 => "NOW".into(),
        60..=3599 => format!("{}M", secs / 60),
        3600..=86_399 => format!("{}H", secs / 3600),
        _ => format!("{}D", secs / 86_400),
    }
}

/// True when `pid` is still a `claude` process on `tty`, per a fresh `ps` output. A tty that
/// closed and was reused by another tab fails this check.
pub fn is_session_on_tty(ps: &str, pid: u32, tty: &str) -> bool {
    claude_processes(ps)
        .iter()
        .any(|(live_pid, live_tty)| *live_pid == pid && live_tty == tty)
}

/// Build the session list from `(pid, tty, cwd)` triples. `ages` holds the tty age per tty.
/// `owners` maps each tty to the app whose tab output listed it. Unlisted ttys are `Other`.
///
/// Title source, first match wins: the terminal tab title, then the transcript summary for the
/// working directory (`summary_for`), then the directory label, then the tty. The status comes
/// from `status_for`. The list is sorted: active first by label, then stale by most recent.
pub fn build_sessions(
    procs: Vec<(u32, String, Option<String>)>,
    tab_titles: &HashMap<String, String>,
    owners: &HashMap<String, TerminalApp>,
    ages: &HashMap<String, Duration>,
    home: &str,
    summary_for: impl Fn(&str) -> Option<String>,
) -> Vec<ClaudeSession> {
    let mut sessions: Vec<ClaudeSession> = procs
        .into_iter()
        .map(|(pid, tty, cwd)| {
            let age = ages.get(&tty).copied();
            let app = owners.get(&tty).copied().unwrap_or(TerminalApp::Other);
            let titled = tab_titles
                .get(&tty)
                .map(|raw| parse_title(raw))
                .filter(|(_, title)| !title.is_empty());
            let (glyph, label) = match titled {
                Some(found) => found,
                None => (
                    None,
                    match cwd.as_deref() {
                        Some(path) => summary_for(path).unwrap_or_else(|| label_for(path, home)),
                        None => tty.clone(),
                    },
                ),
            };
            let status = status_for(age.unwrap_or(Duration::MAX), glyph);
            ClaudeSession {
                pid,
                app,
                tty,
                label,
                status,
                age,
                focused: false,
            }
        })
        .collect();
    sessions.sort_by(|a, b| {
        order_key(a)
            .cmp(&order_key(b))
            .then_with(|| (&a.label, &a.tty).cmp(&(&b.label, &b.tty)))
    });
    sessions
}

/// Active sessions compare equal, so they sort by label and do not jump on each refresh. Stale
/// sessions sort by age, the most recent first.
fn order_key(session: &ClaudeSession) -> (SessionStatus, Duration) {
    match session.status {
        SessionStatus::Active => (SessionStatus::Active, Duration::ZERO),
        SessionStatus::Stale => (SessionStatus::Stale, session.age.unwrap_or(Duration::MAX)),
    }
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
