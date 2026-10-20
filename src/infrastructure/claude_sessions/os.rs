//! Process spawns for `ps`, `lsof`, `osascript` and `open`. Excluded from the coverage gate; the
//! logic that decides what to do with the output lives in `claude_sessions.rs`.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::{
    build_sessions, claude_processes, cwds_from_lsof, is_valid_tty, parse_tab_titles,
    summary_from_jsonl,
};
use crate::domain::actions::{ClaudeSession, TerminalApp};
use crate::infrastructure::process::run;

mod terminal;
pub use terminal::{focus_session, open_new_session, send_prompt};

/// Budgets for one discovery pass, which takes at most about 0.9 s: ps, then one lsof, then
/// the two terminal apps in parallel.
const PS_TIMEOUT: Duration = Duration::from_millis(150);
const LSOF_TIMEOUT: Duration = Duration::from_millis(250);
// ponytail: iTerm2 took about 380 ms live. Runs off the device loop, so a generous budget is safe.
const TITLE_TIMEOUT: Duration = Duration::from_millis(1500);

/// Hard limit for one `osascript` focus call. Terminal apps can be slow to answer when they activate.
const FOCUS_TIMEOUT: Duration = Duration::from_secs(5);

/// Tab titles from iTerm2, one `<device>\t<name>` line per session. Asked only when iTerm2 runs,
/// so this never launches it.
const ITERM_TITLES: &str = r#"if application "iTerm2" is running then
  tell application "iTerm2"
    set out to ""
    repeat with w in windows
      repeat with t in tabs of w
        repeat with s in sessions of t
          set out to out & (tty of s) & (character id 9) & (name of s) & linefeed
        end repeat
      end repeat
    end repeat
    return out
  end tell
end if
return """#;

/// Tab titles from Terminal.app, one `<device>\t<custom title>` line per tab. Same guard as iTerm2.
const TERMINAL_TITLES: &str = r#"if application "Terminal" is running then
  tell application "Terminal"
    set out to ""
    repeat with w in windows
      repeat with t in tabs of w
        set out to out & (tty of t) & (character id 9) & (custom title of t) & linefeed
      end repeat
    end repeat
    return out
  end tell
end if
return """#;

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
    let owners = terminal_titles
        .keys()
        .map(|tty| (tty.clone(), TerminalApp::Terminal))
        .chain(
            iterm_titles
                .keys()
                .map(|tty| (tty.clone(), TerminalApp::ITerm)),
        )
        .collect::<HashMap<_, _>>();
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
    build_sessions(entries, &tab_titles, &owners, &ages, &home, |cwd| {
        summary_for(cwd, &home)
    })
}

/// Time since the tty last printed. The device's modification time moves on every write to the
/// terminal. `None` when the device cannot be read.
fn tty_age(tty: &str) -> Option<Duration> {
    let modified = std::fs::metadata(format!("/dev/{tty}"))
        .ok()?
        .modified()
        .ok()?;
    Some(
        SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default(),
    )
}

/// The session name from the newest transcript for `cwd` under `~/.claude/projects`. Claude Code
/// stores transcripts in a folder named after the path with every non-alphanumeric as `-`.
fn summary_for(cwd: &str, home: &str) -> Option<String> {
    let encoded: String = cwd
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let dir = Path::new(home).join(".claude/projects").join(encoded);
    let newest = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
        .max_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok())?;
    summary_from_jsonl(&std::fs::read_to_string(newest.path()).ok()?)
}
