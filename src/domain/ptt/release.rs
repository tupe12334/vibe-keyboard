//! Key up. Ends the pending begin or the active hold that belongs to a key, or every hold at once.

use tracing::info;

use super::guard::PttGuard;
use super::repeat::Beats;
use super::{KeyEvent, KeyPoster};

/// Why a hold ended. Written to the key-up log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    /// The user let go of the key.
    Key,
    /// Focus left the hold's app for two probes in a row. `to` is the new app, if known.
    Moved { to: Option<i32> },
    /// Accessibility is off.
    Untrusted,
    /// The app did not touch the guard for the watchdog time.
    Watchdog,
    /// The screen changed.
    Screen,
    /// The device disconnected.
    Disconnected,
    /// The app is stopping.
    Shutdown,
}

impl Release {
    /// The reason as the key-up log shows it.
    pub fn label(self) -> String {
        match self {
            Self::Key => "released".into(),
            Self::Moved { to: Some(pid) } => format!("moved to pid {pid}"),
            Self::Moved { to: None } => "moved, no frontmost app".into(),
            Self::Untrusted => "untrusted".into(),
            Self::Watchdog => "watchdog".into(),
            Self::Screen => "screen change".into(),
            Self::Disconnected => "device lost".into(),
            Self::Shutdown => "shutdown".into(),
        }
    }
}

/// The key-up log line: repeat count, reason, widest repeat gap and late beats.
pub(super) fn key_up_summary(beats: Beats, reason: Release) -> String {
    format!(
        "key up after {} repeats: {}, max repeat gap {} ms, late beats {}",
        beats.count,
        reason.label(),
        beats.max_gap.as_millis(),
        beats.late
    )
}

impl<P: KeyPoster> PttGuard<P> {
    /// Key up for `key`. Ends only the pending begin or the active hold that belong to `key`.
    pub fn end(&mut self, key: u8) {
        if self.pending.is_some_and(|(pending, _)| pending == key) {
            self.pending = None;
        }
        if self.hold.as_ref().is_some_and(|hold| hold.key == key) {
            self.release(Release::Key);
        }
    }

    /// Lift the key now and cancel any pending begin. Used on shutdown, disconnect, page change
    /// and lost Accessibility.
    pub fn release_all(&mut self, reason: Release) {
        self.pending = None;
        self.release(reason);
    }

    /// Post the key up for the active hold, to the pid it started on, and log why and how it went.
    pub(super) fn release(&mut self, reason: Release) {
        if let Some(hold) = self.hold.take() {
            info!("push to talk: {}", key_up_summary(hold.beats, reason));
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
        self.release(Release::Shutdown);
    }
}

#[cfg(test)]
#[path = "release_tests.rs"]
mod tests;
