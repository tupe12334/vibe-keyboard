//! Focus and Accessibility checks for a hold, read by the probe thread every probe interval.

use super::guard::PttGuard;
use super::{KeyPoster, Probe, Release};

/// Probes in a row that name another app, or none, before the hold moves. One odd read must not end dictation.
const MOVE_AFTER: u8 = 2;

/// Counts probes in a row that do not name the hold's app.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct MoveCheck {
    misses: u8,
}

impl MoveCheck {
    /// Record one probe. True when the hold's app missed focus for `MOVE_AFTER` probes in a row.
    pub(super) fn observe(&mut self, front: Option<i32>, pid: i32, session_moved: bool) -> bool {
        if front == Some(pid) && !session_moved {
            self.misses = 0;
        } else {
            self.misses = self.misses.saturating_add(1);
        }
        self.misses >= MOVE_AFTER
    }
}

impl<P: KeyPoster> PttGuard<P> {
    /// One OS probe for hold `generation`. Ends the hold when Accessibility is off, or when focus
    /// moved for `MOVE_AFTER` probes in a row. Returns false when the probe thread must stop.
    pub fn observe(&mut self, generation: u64, probe: Probe) -> bool {
        let Some(hold) = &mut self.hold else {
            return false;
        };
        if hold.generation != generation {
            return false;
        }
        let moved = hold
            .moves
            .observe(probe.front, hold.pid, probe.session_moved);
        if !probe.trusted {
            self.release(Release::Untrusted);
            return false;
        }
        if moved {
            self.release(Release::Moved { to: probe.front });
            return false;
        }
        true
    }
}

#[cfg(test)]
#[path = "focus_tests.rs"]
mod tests;
