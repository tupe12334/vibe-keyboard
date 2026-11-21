//! The OS reads of one terminal check, behind a trait so the gate's read order can be tested.
//! Excluded from the coverage gate, like `session_gate.rs`.

use std::time::Duration;

use crate::infrastructure::claude_sessions::tty_landed_within;
use crate::infrastructure::frontmost::{frontmost_bundle_id_within, frontmost_pid_within};

/// The reads of one terminal check. `Os` runs the real reads. Tests fake them.
pub(super) trait FrontReads {
    fn front_pid(&self, limit: Duration) -> Option<i32>;
    fn front_bundle(&self, limit: Duration) -> Option<String>;
    fn tty_shown(&self, index: usize, tty: &str, limit: Duration) -> bool;
}

pub(super) struct Os;

impl FrontReads for Os {
    fn front_pid(&self, limit: Duration) -> Option<i32> {
        frontmost_pid_within(limit)
    }
    fn front_bundle(&self, limit: Duration) -> Option<String> {
        frontmost_bundle_id_within(limit)
    }
    fn tty_shown(&self, index: usize, tty: &str, limit: Duration) -> bool {
        tty_landed_within(index, tty, limit)
    }
}
