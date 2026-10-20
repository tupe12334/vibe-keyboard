//! The hold API: the one guard, and the start of a hold's two threads.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Instant;

use tracing::error;

use super::beat::{beat, watch_focus};
use super::post::{trusted, CgPoster};
use crate::domain::hotkey::{key_code, modifier_flags, Hotkey};
use crate::domain::ptt::{Probe, PttGuard, Release};
use crate::infrastructure::frontmost::frontmost_pid;

/// The one push-to-talk hold of this process. A `static` never drops, so every exit path
/// calls `release_all`, and the panic hook covers panics.
static GUARD: Mutex<PttGuard<CgPoster>> = Mutex::new(PttGuard::new(CgPoster));

/// Reserve a hold for logical `key`. The returned generation goes to `push_to_talk_start`.
pub fn hold_begin(key: u8) -> u64 {
    lock().begin(key)
}

/// Key up for logical `key`. Ends only the hold or pending start that belongs to `key`.
pub fn hold_end(key: u8) {
    lock().end(key);
}

/// Key down for the hold reserved by `hold_begin`, then repeats until `hold_end` or a release.
/// Does nothing if the generation is stale, another hold is active, no app has focus, or
/// Accessibility is off. The repeats and the focus checks run on their own threads.
pub fn push_to_talk_start(generation: u64, key: u8, hotkey: &Hotkey) {
    let Some(code) = key_code(&hotkey.key) else {
        error!("push to talk: no key code for this key");
        return;
    };
    let flags = modifier_flags(&hotkey.modifiers);
    // Read the OS before taking the lock.
    let probe = read_probe();
    if !lock().start(generation, key, code, flags, probe, Instant::now()) {
        return;
    }
    thread::spawn(move || beat(generation));
    thread::spawn(move || watch_focus(generation));
}

/// The app is alive. Called from the device loop, so a stalled loop stops the repeats.
pub fn hold_touch() {
    lock().touch(Instant::now());
}

/// Lift any held key now, logging `reason`. Called on shutdown, disconnect and page change.
pub fn release_all(reason: Release) {
    lock().release_all(reason);
}

/// Lock the guard. A poisoned lock still holds valid state, so use it.
pub(super) fn lock() -> MutexGuard<'static, PttGuard<CgPoster>> {
    GUARD.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the guard needs from the OS. Read outside the lock.
pub(super) fn read_probe() -> Probe {
    Probe {
        front: frontmost_pid(),
        trusted: trusted(),
    }
}
