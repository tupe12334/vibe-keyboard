//! Lifts a held key when the process panics. Reads atomics, never the guard, so a panic
//! while the guard is locked cannot deadlock.

use std::sync::atomic::{AtomicI32, AtomicU16, AtomicU64, Ordering};

use super::post::post_event;

/// Target pid of the key down that has no key up yet, and its key. Zero pid means none.
/// Written by `CgPoster`, read by the panic hook, so the hook never takes `GUARD`.
pub(super) static HELD_PID: AtomicI32 = AtomicI32::new(0);
pub(super) static HELD_CODE: AtomicU16 = AtomicU16::new(0);
pub(super) static HELD_FLAGS: AtomicU64 = AtomicU64::new(0);

/// Lift a held key if the process panics. Reads the atomics, not `GUARD`, so a panic while the
/// guard is locked on this thread cannot deadlock.
pub fn install_panic_release() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let pid = HELD_PID.swap(0, Ordering::SeqCst);
        if pid != 0 {
            post_event(
                Some(pid),
                HELD_CODE.load(Ordering::SeqCst),
                HELD_FLAGS.load(Ordering::SeqCst),
                false,
                false,
            );
        }
        previous(info);
    }));
}
