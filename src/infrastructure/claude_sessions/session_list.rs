use std::collections::HashMap;
use std::time::Duration;

use crate::domain::actions::{ClaudeSession, SessionStatus, TerminalApp};

use super::{label_for, parse_title, status_for};

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
