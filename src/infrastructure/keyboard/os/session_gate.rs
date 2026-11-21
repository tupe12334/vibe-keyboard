//! The key gate for a session hold: the frontmost app must be the terminal that shows the session.
//! Excluded from the coverage gate, like `hold.rs`. The bundle ids live in `claude_sessions/focus_check.rs`.

use std::thread;
use std::time::{Duration, Instant};

use tracing::warn;

use super::session_reads::{FrontReads, Os};
use crate::infrastructure::claude_sessions::{ITERM_BUNDLE, TERMINAL_BUNDLE};

/// Wall-clock budget for the whole gate. Every OS read gets only the time left.
const GATE_DEADLINE: Duration = Duration::from_millis(300);
/// Pause between tries.
const GATE_STEP: Duration = Duration::from_millis(20);

/// Pid of the frontmost terminal when it shows `tty`. None when no terminal shows it within
/// `GATE_DEADLINE`, and the caller posts no key then.
pub(super) fn session_front_pid(tty: &str) -> Option<i32> {
    let start = Instant::now();
    let elapsed = || start.elapsed();
    let landed = first_landed_pid(
        GATE_DEADLINE,
        GATE_STEP,
        &elapsed,
        &|left: &dyn Fn() -> Duration| landed_pid(&Os, tty, left),
        &|step: Duration| thread::sleep(step),
    );
    if landed.is_none() {
        warn!("push to talk: {tty} not frontmost in its terminal, no key posted");
    }
    landed
}

/// Pid of the frontmost app when it is a terminal that shows `tty`. Reads run in this order:
/// pid, bundle, tty, pid. Each read gets `left()`, the time still allowed.
fn landed_pid(reads: &impl FrontReads, tty: &str, left: &dyn Fn() -> Duration) -> Option<i32> {
    let before = reads.front_pid(left())?;
    let index = terminal_index(reads.front_bundle(left()).as_deref())?;
    if !reads.tty_shown(index, tty, left()) {
        return None;
    }
    same_pid(before, reads.front_pid(left()))
}

/// The pid read before the terminal check, when the pid read after it is the same app.
/// A switch in between gives another pid, so no pid is returned.
fn same_pid(before: i32, after: Option<i32>) -> Option<i32> {
    (after == Some(before)).then_some(before)
}

/// Terminal index (0 iTerm2, 1 Terminal.app) of the frontmost bundle id. None for any other app.
fn terminal_index(front: Option<&str>) -> Option<usize> {
    match front {
        Some(ITERM_BUNDLE) => Some(0),
        Some(TERMINAL_BUNDLE) => Some(1),
        _ => None,
    }
}

/// Calls `landed` until it returns a pid or `budget` runs out, pausing `step` between calls. The
/// pause never runs past the deadline. `elapsed` is the time since the gate began.
fn first_landed_pid(
    budget: Duration,
    step: Duration,
    elapsed: &dyn Fn() -> Duration,
    landed: &dyn Fn(&dyn Fn() -> Duration) -> Option<i32>,
    pause: &dyn Fn(Duration),
) -> Option<i32> {
    let left = || budget.saturating_sub(elapsed());
    loop {
        if left().is_zero() {
            return None;
        }
        if let Some(pid) = landed(&left) {
            return Some(pid);
        }
        let rest = left();
        if rest.is_zero() {
            return None;
        }
        pause(step.min(rest));
    }
}

#[cfg(test)]
#[path = "session_gate_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "session_check_tests.rs"]
mod check_tests;
