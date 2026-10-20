//! Key down that starts a hold. Refused when the generation is stale, another hold is active,
//! no app has focus, or Accessibility is off.

use std::time::Instant;

use tracing::warn;

use super::focus::MoveCheck;
use super::guard::{Hold, PttGuard};
use super::repeat::Beats;
use super::{KeyEvent, KeyPoster, Probe};

impl<P: KeyPoster> PttGuard<P> {
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
            moves: MoveCheck::default(),
            beats: Beats::new(now),
        });
        true
    }
}
