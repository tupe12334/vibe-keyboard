//! The hold's state, and the key down that starts it.

use super::focus::MoveCheck;
use super::repeat::Beats;
use super::KeyPoster;
use std::time::Instant;

pub(super) struct Hold {
    pub(super) key: u8,
    pub(super) code: u16,
    pub(super) flags: u64,
    pub(super) pid: i32,
    pub(super) generation: u64,
    pub(super) touched: Instant,
    pub(super) moves: MoveCheck,
    pub(super) beats: Beats,
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
}

#[cfg(test)]
#[path = "guard_tests.rs"]
mod tests;
