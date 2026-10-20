//! Owns the one push-to-talk hold. Pure: the OS poster and the probes come from the caller.
//! Key down, repeats and key up all go to one target pid, captured before the first key down.
//! Every key event goes through here, so no path can leave Space held down.

use std::time::Duration;

mod focus;
mod guard;
mod release;
mod repeat;
mod start;
#[cfg(test)]
mod support;

pub use guard::PttGuard;
pub use release::Release;
pub use repeat::{deadline, next_index, REPEAT_EVERY};

/// A hold lifts its key when the app has not touched the guard for this long.
pub const WATCHDOG: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: u16,
    pub flags: u64,
    pub down: bool,
    pub repeat: bool,
}

/// What the OS reported just before a decision. Read outside the guard's lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Probe {
    /// Pid of the frontmost app, the one that gets typed keys.
    pub front: Option<i32>,
    /// Whether this process may post key events.
    pub trusted: bool,
    /// The selected session's tty is no longer iTerm2's front session.
    pub session_moved: bool,
}

/// Posts a key event to one process.
pub trait KeyPoster {
    fn post(&mut self, pid: i32, event: KeyEvent);
}
