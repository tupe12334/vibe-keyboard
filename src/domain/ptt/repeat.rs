//! The repeat tick and the watchdog. A hold stops itself when the app goes quiet.

use std::time::{Duration, Instant};

use super::guard::PttGuard;
use super::{KeyEvent, KeyPoster, Release, WATCHDOG};

/// Repeat count and the widest gap between two repeats of one hold, for the key-up log.
#[derive(Debug, Clone, Copy)]
pub(super) struct Beats {
    pub(super) count: u32,
    last: Instant,
    pub(super) max_gap: Duration,
}

impl Beats {
    /// Start counting at the key down `now`.
    pub(super) const fn new(now: Instant) -> Self {
        Self {
            count: 0,
            last: now,
            max_gap: Duration::ZERO,
        }
    }

    /// One repeat was posted at `now`.
    pub(super) fn beat(&mut self, now: Instant) {
        self.max_gap = self.max_gap.max(now.duration_since(self.last));
        self.last = now;
        self.count += 1;
    }
}

impl<P: KeyPoster> PttGuard<P> {
    /// The app is alive. Called from the device loop, so a stalled loop stops the repeats.
    pub const fn touch(&mut self, now: Instant) {
        if let Some(hold) = &mut self.hold {
            hold.touched = now;
        }
    }

    /// One repeat tick for hold `generation`. Returns false when the repeat thread must stop.
    /// Ends the hold on a stale touch. Focus and trust are checked by `observe`, off this path.
    pub fn pump(&mut self, generation: u64, now: Instant) -> bool {
        let Some(hold) = &mut self.hold else {
            return false;
        };
        if hold.generation != generation {
            return false;
        }
        if now.duration_since(hold.touched) > WATCHDOG {
            self.release(Release::Watchdog);
            return false;
        }
        hold.beats.beat(now);
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
