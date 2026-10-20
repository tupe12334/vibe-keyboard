//! The repeat tick and the watchdog. A hold stops itself when the app goes quiet.

use std::time::{Duration, Instant};

use super::guard::PttGuard;
use super::{KeyEvent, KeyPoster, Release, WATCHDOG};

/// Gap between repeated key downs while held. Terminal key repeat is about 30 ms.
pub const REPEAT_EVERY: Duration = Duration::from_millis(30);
/// A gap wider than this counts as a late beat in the key-up log.
pub const LATE_AFTER: Duration = Duration::from_millis(50);

/// Deadline of repeat `index` (1-based) on the grid that starts at `start`. Never drifts.
pub fn deadline(start: Instant, index: u32) -> Instant {
    start + REPEAT_EVERY * index
}

/// The next repeat index to wait for, after repeat `index` was posted at `now`. Grid points
/// already passed are skipped, so a late wake posts once and never bursts to catch up.
pub fn next_index(start: Instant, index: u32, now: Instant) -> u32 {
    let elapsed = now.saturating_duration_since(start).as_nanos() / REPEAT_EVERY.as_nanos();
    index.max(u32::try_from(elapsed).unwrap_or(u32::MAX)) + 1
}

/// Repeat count, late beats and the widest gap between two repeats of one hold, for the key-up log.
#[derive(Debug, Clone, Copy)]
pub(super) struct Beats {
    pub(super) count: u32,
    pub(super) late: u32,
    last: Instant,
    pub(super) max_gap: Duration,
}

impl Beats {
    /// Start counting at the key down `now`.
    pub(super) const fn new(now: Instant) -> Self {
        Self {
            count: 0,
            late: 0,
            last: now,
            max_gap: Duration::ZERO,
        }
    }

    /// One repeat was posted at `now`.
    pub(super) fn beat(&mut self, now: Instant) {
        let gap = now.duration_since(self.last);
        self.max_gap = self.max_gap.max(gap);
        if gap > LATE_AFTER {
            self.late += 1;
        }
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
