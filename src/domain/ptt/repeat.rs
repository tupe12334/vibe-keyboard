//! The repeat tick and the watchdog. A hold stops itself when the app goes quiet or focus moves.

use std::time::Instant;

use super::guard::PttGuard;
use super::{KeyEvent, KeyPoster, Probe, WATCHDOG};

impl<P: KeyPoster> PttGuard<P> {
    /// The app is alive. Called from the device loop, so a stalled loop stops the repeats.
    pub const fn touch(&mut self, now: Instant) {
        if let Some(hold) = &mut self.hold {
            hold.touched = now;
        }
    }

    /// One repeat tick for hold `generation`. Returns false when the repeat thread must stop.
    /// Ends the hold (key up to its own pid) on a stale touch, a focus change or lost trust.
    pub fn pump(&mut self, generation: u64, probe: Probe, now: Instant) -> bool {
        let Some(hold) = &self.hold else {
            return false;
        };
        if hold.generation != generation {
            return false;
        }
        let stale = now.duration_since(hold.touched) > WATCHDOG;
        let moved = probe.front != Some(hold.pid);
        if stale || moved || !probe.trusted {
            self.release();
            return false;
        }
        let pid = hold.pid;
        let event = KeyEvent {
            code: hold.code,
            flags: hold.flags,
            down: true,
            repeat: true,
        };
        self.poster.post(pid, event);
        true
    }
}

#[cfg(test)]
#[path = "repeat_tests.rs"]
mod tests;
