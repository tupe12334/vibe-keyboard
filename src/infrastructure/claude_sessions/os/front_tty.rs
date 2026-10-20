//! osascript reads of the tty shown in front, the tab scan, the wait for focus to land, and the
//! BLINK highlight. Excluded from the coverage gate; rules live in `focus_check.rs`, `highlight.rs`.

use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use tracing::warn;

use super::tab_scripts::{ITERM_FRONT_TTY, TERMINAL_FRONT_TTY};
use super::terminal::focus_session;
use super::FOCUS_TIMEOUT;
use crate::domain::actions::TerminalApp;
use crate::infrastructure::claude_sessions::{
    highlight_target, is_selected, scan_result, session_moved, Scan,
};
use crate::infrastructure::process::run;

/// Hard limit for one read. Each read is a separate osascript call.
const READ_TIMEOUT: Duration = Duration::from_secs(2);
/// How long focus may take to land on the selected tty before dictation is refused.
const SETTLE: Duration = Duration::from_secs(1);
const POLL: Duration = Duration::from_millis(50);

/// The tty shown in front by the terminal at `index` (0 iTerm2, 1 Terminal.app). None when unknown.
fn front_tty(index: usize) -> Option<String> {
    let script = [ITERM_FRONT_TTY, TERMINAL_FRONT_TTY].get(index)?;
    let out = run("osascript", &["-e", script], READ_TIMEOUT)?;
    Some(out.trim().to_string()).filter(|tty| !tty.is_empty())
}

const SCAN_ATTEMPTS: usize = 3;
const SCAN_RETRY_WAIT: Duration = Duration::from_millis(150);

/// Index of the script (0 iTerm2, 1 Terminal.app) whose terminal holds the tab, or None. A failed
/// scan (iTerm2 reorders its windows mid-scan, then prints nothing) is retried. Missing is not.
pub(super) fn find_tab(scripts: &[String; 2]) -> Option<usize> {
    for attempt in 0..SCAN_ATTEMPTS {
        if attempt > 0 {
            thread::sleep(SCAN_RETRY_WAIT);
        }
        let mut failed = false;
        for (index, script) in scripts.iter().enumerate() {
            match scan_result(run("osascript", &["-e", script], FOCUS_TIMEOUT).as_deref()) {
                Scan::Found => return Some(index),
                Scan::Missing => {}
                Scan::Failed => failed = true,
            }
        }
        if !failed {
            return None;
        }
    }
    None
}

/// Poll for up to `SETTLE` until the terminal at `index` shows `tty` in front. Logs a warning and
/// returns false when it never does, so the caller posts no key.
pub fn settle_on(index: usize, tty: &str) -> bool {
    let deadline = Instant::now() + SETTLE;
    loop {
        let got = front_tty(index);
        if is_selected(got.as_deref(), tty) {
            return true;
        }
        if Instant::now() >= deadline {
            warn!(
                "push to talk: selected session tty {tty} not frontmost (got {})",
                got.as_deref().unwrap_or("unknown")
            );
            return false;
        }
        thread::sleep(POLL);
    }
}

/// True when iTerm2's current session is no longer `tty`. Read once, for the push-to-talk probe.
pub fn iterm_moved_from(tty: &str) -> bool {
    session_moved(front_tty(0).as_deref(), tty)
}

/// Focus the session on `tty`, then ask iTerm2 for a fireworks highlight on that pane. Only iTerm2
/// reads the escape, so other apps get focus only. Blocks; call it from a thread.
pub fn blink_session(tty: &str, app: TerminalApp) {
    focus_session(tty);
    if app != TerminalApp::ITerm {
        return;
    }
    let Some((path, bytes)) = highlight_target(tty) else {
        return;
    };
    let written = OpenOptions::new()
        .write(true)
        .open(&path)
        .and_then(|mut device| device.write_all(bytes));
    if let Err(err) = written {
        warn!("claude sessions: highlight on {path} failed: {err}");
    }
}
