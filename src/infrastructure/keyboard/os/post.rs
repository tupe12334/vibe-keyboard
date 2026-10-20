//! CoreGraphics key posting and the Accessibility check. Excluded from the coverage gate.

use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, EventField};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use foreign_types::ForeignType;
use tracing::{error, info, warn};

use super::accessibility::accessibility_trusted;
use super::panic::{HELD_CODE, HELD_FLAGS, HELD_PID};
use crate::domain::hotkey::{key_code, modifier_flags, Hotkey};
use crate::domain::ptt::{KeyEvent, KeyPoster};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventPostToPid(pid: i32, event: *mut c_void);
}

/// Repeats sent for the current hold, for the key-up log line.
static REPEATS: AtomicU32 = AtomicU32::new(0);

/// The OS side of `KeyPoster`: CoreGraphics posts, and it records the key down it sent so the
/// panic hook can lift it.
pub struct CgPoster;

impl KeyPoster for CgPoster {
    fn post(&mut self, pid: i32, event: KeyEvent) {
        if event.down && !event.repeat {
            HELD_CODE.store(event.code, Ordering::SeqCst);
            HELD_FLAGS.store(event.flags, Ordering::SeqCst);
            HELD_PID.store(pid, Ordering::SeqCst);
            REPEATS.store(0, Ordering::SeqCst);
            info!("push to talk: key down for pid {pid}");
        } else if event.repeat {
            REPEATS.fetch_add(1, Ordering::SeqCst);
        } else if !event.down {
            HELD_PID.store(0, Ordering::SeqCst);
            let repeats = REPEATS.swap(0, Ordering::SeqCst);
            info!("push to talk: key up after {repeats} repeats");
        }
        // The HID tap reaches the frontmost app like a real keyboard, repeats included. Focus
        // already verified that the session's terminal is frontmost (#220).
        post_event(None, event.code, event.flags, event.down, event.repeat);
    }
}

/// Whether this process may post key events. Logs the fix when it may not.
pub(super) fn trusted() -> bool {
    let ok = accessibility_trusted();
    if !ok {
        warn!(
            "push to talk needs Accessibility for this process: System Settings > Privacy & Security > Accessibility, then add or enable the app that runs listen (see README)"
        );
    }
    ok
}

/// One press and release of `hotkey`, to the frontmost app.
pub fn tap(hotkey: &Hotkey) {
    let Some(code) = key_code(&hotkey.key) else {
        error!("push to talk: no key code for this key");
        return;
    };
    if !trusted() {
        return;
    }
    let flags = modifier_flags(&hotkey.modifiers);
    post_event(None, code, flags, true, false);
    post_event(None, code, flags, false, false);
}

/// Post a key event. `None` goes through the HID tap to the frontmost app; `Some(pid)` goes to
/// that process only.
pub(super) fn post_event(target: Option<i32>, code: u16, flags: u64, down: bool, repeat: bool) {
    let Ok(source) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) else {
        return;
    };
    let Ok(event) = CGEvent::new_keyboard_event(source, code, down) else {
        return;
    };
    event.set_flags(CGEventFlags::from_bits_truncate(flags));
    if repeat {
        event.set_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT, 1);
    }
    match target {
        Some(pid) => {
            // SAFETY: `event` is a live CGEvent for the duration of this call.
            unsafe { CGEventPostToPid(pid, event.as_ptr().cast()) }
        }
        None => event.post(CGEventTapLocation::HID),
    }
}
