//! CoreGraphics left-button check and release. Excluded from the coverage gate.
//! The decision lives in `mouse_release.rs`.

/// Read the left button after `CLEAR_DELAY`, and release it once if it is held.
/// Runs on its own thread, so the connect path does not wait.
#[cfg(target_os = "macos")]
pub fn clear_stuck_left_button() {
    use super::{should_clear, CLEAR_DELAY};
    use tracing::warn;

    std::thread::spawn(|| {
        std::thread::sleep(CLEAR_DELAY);
        if should_clear(left_down()) {
            post_left_up();
            warn!("cleared stuck left mouse button after deck connect");
        }
    });
}

/// Other platforms have no stuck mouse button to clear.
#[cfg(not(target_os = "macos"))]
pub fn clear_stuck_left_button() {}

/// Whether the left button is down in the HID system state.
#[cfg(target_os = "macos")]
fn left_down() -> bool {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
    }
    // kCGEventSourceStateHIDSystemState = 1, kCGMouseButtonLeft = 0.
    // SAFETY: takes two plain integers and only reads the HID button state.
    unsafe { CGEventSourceButtonState(1, 0) }
}

/// Post one left mouse-up at the current cursor location, through the HID tap.
#[cfg(target_os = "macos")]
fn post_left_up() {
    use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let source = || CGEventSource::new(CGEventSourceStateID::HIDSystemState);
    let Ok(probe) = source().and_then(CGEvent::new) else {
        return;
    };
    let at = probe.location();
    let Ok(source) = source() else {
        return;
    };
    if let Ok(up) =
        CGEvent::new_mouse_event(source, CGEventType::LeftMouseUp, at, CGMouseButton::Left)
    {
        up.post(CGEventTapLocation::HID);
    }
}
