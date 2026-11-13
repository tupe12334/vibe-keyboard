//! The key gate for a session hold: the frontmost app must be the terminal that shows the session.
//! Excluded from the coverage gate, like `hold.rs`. The rules live in `claude_sessions/focus_check.rs`.

use std::thread;
use std::time::Duration;

use tracing::warn;

use crate::infrastructure::claude_sessions::{first_landed_pid, terminal_index, tty_landed};
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
