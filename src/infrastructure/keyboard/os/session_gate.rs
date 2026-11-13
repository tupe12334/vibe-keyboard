//! The key gate for a session hold: the frontmost app must be the terminal that shows the session.
//! Excluded from the coverage gate, like `hold.rs`. The bundle ids live in `claude_sessions/focus_check.rs`.

use std::thread;
use std::time::Duration;

use tracing::warn;

use crate::infrastructure::claude_sessions::{tty_landed, ITERM_BUNDLE, TERMINAL_BUNDLE};
use crate::infrastructure::frontmost::{frontmost_bundle_id, frontmost_pid};

/// Gate: 15 tries, 20 ms apart, so about 300 ms plus the time of each read.
const GATE_ATTEMPTS: usize = 15;
const GATE_STEP: Duration = Duration::from_millis(20);

/// Pid of the frontmost terminal when it shows `tty`. None when no terminal shows it in time, and
/// the caller posts no key then.
pub(super) fn session_front_pid(tty: &str) -> Option<i32> {
    let landed = first_landed_pid(
        GATE_ATTEMPTS,
        || landed_pid(tty),
        || {
            thread::sleep(GATE_STEP);
        },
    );
    if landed.is_none() {
        warn!("push to talk: {tty} not frontmost in its terminal, no key posted");
    }
    landed
}

/// Pid of the frontmost app when it is a terminal that shows `tty`. The pid is read again after
/// the tty check, so a switch in between fails this read.
fn landed_pid(tty: &str) -> Option<i32> {
    let pid = frontmost_pid()?;
    let index = terminal_index(frontmost_bundle_id().as_deref())?;
    (tty_landed(index, tty) && frontmost_pid() == Some(pid)).then_some(pid)
}

/// Terminal index (0 iTerm2, 1 Terminal.app) of the frontmost bundle id. None for any other app.
fn terminal_index(front: Option<&str>) -> Option<usize> {
    match front {
        Some(ITERM_BUNDLE) => Some(0),
        Some(TERMINAL_BUNDLE) => Some(1),
        _ => None,
    }
}

/// Calls `landed` until it returns a pid, at most `attempts` times, pausing between calls.
/// None when no call landed. The caller posts no key then.
fn first_landed_pid(
    attempts: usize,
    mut landed: impl FnMut() -> Option<i32>,
    mut pause: impl FnMut(),
) -> Option<i32> {
    for attempt in 0..attempts {
        if attempt > 0 {
            pause();
        }
        if let Some(pid) = landed() {
            return Some(pid);
        }
    }
    None
}

#[cfg(test)]
#[path = "session_gate_tests.rs"]
mod tests;
