//! Key up. Ends the pending begin or the active hold that belongs to a key, or every hold at once.

use super::guard::PttGuard;
use super::{KeyEvent, KeyPoster};

impl<P: KeyPoster> PttGuard<P> {
    /// Key up for `key`. Ends only the pending begin or the active hold that belong to `key`.
    pub fn end(&mut self, key: u8) {
        if self.pending.is_some_and(|(pending, _)| pending == key) {
            self.pending = None;
        }
        if self.hold.as_ref().is_some_and(|hold| hold.key == key) {
            self.release();
        }
    }

    /// Lift the key now and cancel any pending begin. Used on shutdown, disconnect, page change
    /// and lost Accessibility.
    pub fn release_all(&mut self) {
        self.pending = None;
        self.release();
    }

    /// Post the key up for the active hold, to the pid it started on.
    pub(super) fn release(&mut self) {
        if let Some(hold) = self.hold.take() {
            self.poster.post(
                hold.pid,
                KeyEvent {
                    code: hold.code,
                    flags: hold.flags,
                    down: false,
                    repeat: false,
                },
            );
        }
    }
}

// Only unit tests drop a guard. The app keeps its guard in a `static`, which never drops; its
// exit paths call `release_all` and the panic hook posts the key up without the lock.
impl<P: KeyPoster> Drop for PttGuard<P> {
    fn drop(&mut self) {
        self.release();
    }
}

#[cfg(test)]
#[path = "release_tests.rs"]
mod tests;
