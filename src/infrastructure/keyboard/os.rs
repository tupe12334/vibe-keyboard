//! Push-to-talk hold API for the device loop. Excluded from the coverage gate.
//!
//! Claude Code starts dictation on a held key by counting rapid key-repeat events. A real
//! keyboard sends those repeats; a single synthetic key down does not. So a hold posts key
//! down again, with the auto-repeat flag set, until the hold ends. The hold rules live in
//! `domain/ptt.rs`; this file only probes the OS and posts.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use crate::domain::hotkey::{key_code, modifier_flags, Hotkey};
use crate::domain::ptt::{Probe, PttGuard};
use tracing::error;

mod panic;
mod post;
mod window;

pub use panic::install_panic_release;
pub use post::tap;

use post::{trusted, CgPoster};
use window::front_window_pid;

/// Gap between repeated key downs while held. Terminal key repeat is about 30 ms.
const REPEAT_EVERY: Duration = Duration::from_millis(30);

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
/// The OS is read before the lock is taken. Does nothing if the generation is stale, another
/// hold is active, no app has focus, or Accessibility is off.
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
    thread::spawn(move || loop {
        thread::sleep(REPEAT_EVERY);
        let probe = read_probe();
        if !lock().pump(generation, probe, Instant::now()) {
            break;
        }
    });
}

/// The app is alive. Called from the device loop, so a stalled loop stops the repeats.
pub fn hold_touch() {
    lock().touch(Instant::now());
}

/// Lift any held key now. Called on shutdown, disconnect and page change.
pub fn release_all() {
    lock().release_all();
}

/// Lock the guard. A poisoned lock still holds valid state, so use it.
fn lock() -> MutexGuard<'static, PttGuard<CgPoster>> {
    GUARD.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the guard needs from the OS. Read outside the lock.
fn read_probe() -> Probe {
    Probe {
        front: front_window_pid(),
        trusted: trusted(),
    }
}
