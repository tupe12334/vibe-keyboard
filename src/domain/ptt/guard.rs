//! The hold's state, and the key down that starts it.

use std::time::Instant;

use tracing::warn;

use super::{KeyEvent, KeyPoster, Probe};

pub(super) struct Hold {
    pub(super) key: u8,
    pub(super) code: u16,
    pub(super) flags: u64,
    pub(super) pid: i32,
    pub(super) generation: u64,
    pub(super) touched: Instant,
}

/// The one hold of a process. Every event goes through its poster.
pub struct PttGuard<P: KeyPoster> {
    pub(super) poster: P,
    pub(super) generation: u64,
    /// The key and generation of the last `begin`, until its `start` or its key up.
    pub(super) pending: Option<(u8, u64)>,
    pub(super) hold: Option<Hold>,
}

impl<P: KeyPoster> PttGuard<P> {
    pub const fn new(poster: P) -> Self {
        Self {
            poster,
            generation: 0,
            pending: None,
            hold: None,
        }
    }

    /// Reserve a hold for `key`. Returns its generation. Pass it to `start`.
    pub const fn begin(&mut self, key: u8) -> u64 {
        self.generation += 1;
        self.pending = Some((key, self.generation));
        self.generation
    }

    /// Key down for the reserved hold, to the frontmost app's pid from `probe`. Refused when the
    /// generation is stale, another hold is active, no app has focus, or Accessibility is off.
    pub fn start(
        &mut self,
        generation: u64,
        key: u8,
        code: u16,
        flags: u64,
        probe: Probe,
        now: Instant,
    ) -> bool {
        if self.pending != Some((key, generation)) {
            return false;
        }
        self.pending = None;
        if let Some(active) = &self.hold {
            warn!(
                "push to talk: key {key} ignored, key {} already holds it",
                active.key
            );
            return false;
        }
        let Some(pid) = probe.front else {
            return false;
        };
        if !probe.trusted {
            return false;
        }
        self.poster.post(
            pid,
            KeyEvent {
                code,
                flags,
                down: true,
                repeat: false,
            },
        );
        self.hold = Some(Hold {
            key,
            code,
            flags,
            pid,
            generation,
            touched: now,
        });
        true
    }
}

#[cfg(test)]
#[path = "guard_tests.rs"]
mod tests;
