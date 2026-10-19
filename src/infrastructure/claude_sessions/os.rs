//! Process spawns for `ps`, `lsof`, `osascript` and `open`. Excluded from the coverage gate; the
//! logic that decides what to do with the output lives in `claude_sessions.rs`.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::{
    build_sessions, claude_processes, cwds_from_lsof, focus_scripts, parse_tab_titles,
    summary_from_jsonl,
};
use crate::domain::actions::ClaudeSession;

/// Budgets for one discovery pass, which takes at most about 0.9 s: ps, then one lsof, then
/// the two terminal apps in parallel.
const PS_TIMEOUT: Duration = Duration::from_millis(150);
const LSOF_TIMEOUT: Duration = Duration::from_millis(250);
const TITLE_TIMEOUT: Duration = Duration::from_millis(500);

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
          set out to out & (tty of s) & tab & (name of s) & linefeed
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
        set out to out & (tty of t) & tab & (custom title of t) & linefeed
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
    let mut tab_titles = run("osascript", &["-e", TERMINAL_TITLES], TITLE_TIMEOUT)
        .map(|out| parse_tab_titles(&out))
        .unwrap_or_default();
    tab_titles.extend(
        iterm
            .join()
            .ok()
            .flatten()
            .map(|out| parse_tab_titles(&out))
            .unwrap_or_default(),
    );

    let home = std::env::var("HOME").unwrap_or_default();
    let entries = procs
        .into_iter()
        .map(|(pid, tty)| (tty, cwds.get(&pid).cloned()))
        .collect();
    build_sessions(entries, &tab_titles, &home, |cwd| summary_for(cwd, &home))
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

/// Select the terminal tab for `tty`. Tries iTerm2, then Terminal.app. When neither has the tab
/// (VS Code and other terminals), it activates VS Code. Blocks until done, so call it from a thread.
pub fn focus_session(tty: &str) {
    let Some(scripts) = focus_scripts(tty) else {
        return;
    };
    let found = scripts.iter().any(|script| {
        run("osascript", &["-e", script], FOCUS_TIMEOUT).is_some_and(|out| out.trim() == "found")
    });
    if !found {
        run("open", &["-b", "com.microsoft.VSCode"], FOCUS_TIMEOUT);
    }
}

/// Run a program with no shell. Returns its stdout when it ends within `timeout`. The exit
/// status is ignored, because `lsof` exits non-zero when one pid has already gone.
fn run(program: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    // Read stdout on its own thread. A full pipe would otherwise stall the child until timeout.
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        stdout.read_to_string(&mut text).map(|_| text)
    });
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    reader.join().ok()?.ok()
}
