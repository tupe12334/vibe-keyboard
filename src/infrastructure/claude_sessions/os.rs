//! Process spawns for `ps`, `lsof`, `osascript` and `open`. Excluded from the coverage gate; the
//! logic that decides what to do with the output lives in `claude_sessions.rs`.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use super::{
    build_sessions, claude_processes, cwds_from_lsof, drop_routines, is_valid_tty,
    mark_remote_control, parse_tab_titles, tab_owners,
};
use crate::domain::actions::ClaudeSession;
use crate::infrastructure::process::run;

mod front_tty;
mod fullscreen;
mod summary;
mod tab_scripts;
mod terminal;
mod tty_age;
pub use front_tty::{blink_session, iterm_moved_from};
pub use fullscreen::full_screen_session;
use summary::summary_for;
use tab_scripts::{ITERM_TITLES, TERMINAL_TITLES};
pub use terminal::{focus_session, open_new_session, send_prompt};
use tty_age::tty_age;

/// Budgets for one discovery pass, which takes at most about 0.9 s: ps, then one lsof, then
/// the two terminal apps in parallel.
const PS_TIMEOUT: Duration = Duration::from_millis(150);
const LSOF_TIMEOUT: Duration = Duration::from_millis(250);
const TMUX_TIMEOUT: Duration = Duration::from_millis(250);
const TMUX_PANES: &[&str] = &["list-panes", "-a", "-F", "#{session_name} #{pane_tty}"];
// ponytail: iTerm2 took about 380 ms live. Runs off the device loop, so a generous budget is safe.
const TITLE_TIMEOUT: Duration = Duration::from_millis(1500);

/// Hard limit for one `osascript` focus call. Terminal apps can be slow to answer when they activate.
const FOCUS_TIMEOUT: Duration = Duration::from_secs(5);

/// Every `claude` process on a terminal, with its title, status and working directory.
pub fn list_sessions() -> Vec<ClaudeSession> {
    let Some(ps) = run("ps", &["-Ao", "pid=,tty=,comm="], PS_TIMEOUT) else {
        return Vec::new();
    };
    let procs = claude_processes(&ps);
    if procs.is_empty() {
        return Vec::new();
    }
    let pids = procs
        .iter()
        .map(|(pid, _)| pid.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let cwds = run(
        "lsof",
        &["-a", "-d", "cwd", "-Fpn", "-p", &pids],
        LSOF_TIMEOUT,
    )
    .map(|out| cwds_from_lsof(&out))
    .unwrap_or_default();
    // Moadim routines are not user sessions. They stay on the Routines page. No tmux means no panes.
    let tmux = run("tmux", TMUX_PANES, TMUX_TIMEOUT).unwrap_or_default();
    let procs = drop_routines(procs, &cwds, &tmux);
    if procs.is_empty() {
        return Vec::new();
    }

    let iterm = std::thread::spawn(|| run("osascript", &["-e", ITERM_TITLES], TITLE_TIMEOUT));
    let terminal_titles = run("osascript", &["-e", TERMINAL_TITLES], TITLE_TIMEOUT)
        .map(|out| parse_tab_titles(&out))
        .unwrap_or_default();
    let iterm_titles = iterm
        .join()
        .ok()
        .flatten()
        .map(|out| parse_tab_titles(&out))
        .unwrap_or_default();
    // The app whose tab output lists a tty owns that tab. Title or not, the tab is there.
    let owners = tab_owners(&terminal_titles, &iterm_titles);
    let mut tab_titles = terminal_titles;
    tab_titles.extend(iterm_titles);

    let home = std::env::var("HOME").unwrap_or_default();
    let ages = procs
        .iter()
        .filter(|(_, tty)| is_valid_tty(tty))
        .filter_map(|(_, tty)| tty_age(tty).map(|age| (tty.clone(), age)))
        .collect::<HashMap<_, _>>();
    let entries = procs
        .into_iter()
        .map(|(pid, tty)| (pid, tty, cwds.get(&pid).cloned()))
        .collect();
    let dir = Path::new(&home).join(".claude/sessions");
    let sessions = build_sessions(entries, &tab_titles, &owners, &ages, &home, |cwd| {
        summary_for(cwd, &home)
    });
    mark_remote_control(sessions, |pid| {
        std::fs::read_to_string(dir.join(format!("{pid}.json"))).ok()
    })
}
