//! Process spawns for `ps`, `lsof` and `osascript`. Excluded from the coverage gate; the
//! logic that decides what to do with the output lives in `claude_sessions.rs`.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::{claude_processes, cwd_from_lsof, focus_scripts, sessions_from};
use crate::domain::actions::ClaudeSession;

/// Hard limit for one `ps` or `lsof` call. A hung call must not stall the device loop.
const TIMEOUT: Duration = Duration::from_secs(1);

/// Hard limit for one `osascript` call. Terminal apps can be slow to answer when they activate.
const FOCUS_TIMEOUT: Duration = Duration::from_secs(5);

/// Every `claude` process on a terminal, with its working directory. Sorted by label.
pub fn list_sessions() -> Vec<ClaudeSession> {
    let Some(ps) = run("ps", &["-Ao", "pid=,tty=,comm="], TIMEOUT) else {
        return Vec::new();
    };
    let home = std::env::var("HOME").unwrap_or_default();
    let entries = claude_processes(&ps)
        .into_iter()
        .map(|(pid, tty)| {
            let pid = pid.to_string();
            let cwd = run("lsof", &["-a", "-p", &pid, "-d", "cwd", "-Fn"], TIMEOUT)
                .and_then(|out| cwd_from_lsof(&out));
            (tty, cwd)
        })
        .collect();
    sessions_from(entries, &home)
}

/// Select the terminal tab for `tty`. Tries iTerm2, then Terminal.app. Blocks until both
/// answer or time out, so call it from a thread.
pub fn focus_session(tty: &str) {
    let Some(scripts) = focus_scripts(tty) else {
        return;
    };
    for script in scripts {
        let found = run("osascript", &["-e", &script], FOCUS_TIMEOUT)
            .is_some_and(|out| out.trim() == "found");
        if found {
            return;
        }
    }
}

/// Run a program with no shell. Returns its stdout when it exits 0 within `timeout`.
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
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    let text = reader.join().ok()?.ok()?;
    status.success().then_some(text)
}
