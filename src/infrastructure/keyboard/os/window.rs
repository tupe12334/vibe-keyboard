//! Looks up the app that owns the topmost normal window.

use std::ffi::c_void;

use core_foundation::base::TCFType;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::number::{CFNumber, CFNumberRef};
use core_foundation::string::CFStringRef;
use core_graphics::window::{
    copy_window_info, kCGNullWindowID, kCGWindowLayer, kCGWindowListExcludeDesktopElements,
    kCGWindowListOptionOnScreenOnly, kCGWindowOwnerPID,
};

/// Pid of the app that owns the topmost normal window. Reads the window list in-process.
pub(super) fn front_window_pid() -> Option<i32> {
    let windows = copy_window_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        kCGNullWindowID,
    )?;
    for index in 0..windows.len() {
        let window = *windows.get(index)?;
        // SAFETY: each item of the window list is a CFDictionary that the list keeps alive.
        let dict = unsafe {
            CFDictionary::<*const c_void, *const c_void>::wrap_under_get_rule(
                window as CFDictionaryRef,
            )
        };
        if window_number(&dict, unsafe { kCGWindowLayer }) == Some(0) {
            return window_number(&dict, unsafe { kCGWindowOwnerPID });
        }
    }
    None
}

fn window_number(
    dict: &CFDictionary<*const c_void, *const c_void>,
    key: CFStringRef,
) -> Option<i32> {
    let value = *dict.find(key as *const c_void)?;
    // SAFETY: the window list stores these values as CFNumber.
    let number = unsafe { CFNumber::wrap_under_get_rule(value as CFNumberRef) };
    number.to_i32()
}
