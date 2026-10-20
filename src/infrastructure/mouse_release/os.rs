//! CoreGraphics left-button check and release. Excluded from the coverage gate.
//! The decision and the log line live in `mouse_release.rs`.

/// Read the left button after `CLEAR_DELAY`, and release it once if it is held.
/// Runs on its own thread, so the connect path does not wait.
#[cfg(target_os = "macos")]
pub fn clear_stuck_left_button() {
    use super::{clear_log_line, should_clear, CLEAR_DELAY};
    use tracing::{info, warn};

    std::thread::spawn(|| {
        std::thread::sleep(CLEAR_DELAY);
        if should_clear(left_down()) {
            let result = post_left_up();
            let line = clear_log_line(result);
            if result.is_ok() {
                info!("{line}");
            } else {
                warn!("{line}");
            }
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
/// `Ok` means the up event was created and posted. `Err` names the step that failed.
#[cfg(target_os = "macos")]
fn post_left_up() -> Result<(), &'static str> {
    use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let source = || CGEventSource::new(CGEventSourceStateID::HIDSystemState);
    let probe = source()
        .and_then(CGEvent::new)
        .map_err(|()| "could not create a cursor probe event")?;
    let at = probe.location();
    let source = source().map_err(|()| "could not create an event source")?;
    let up = CGEvent::new_mouse_event(source, CGEventType::LeftMouseUp, at, CGMouseButton::Left)
        .map_err(|()| "could not create the mouse-up event")?;
    up.post(CGEventTapLocation::HID);
    Ok(())
}
