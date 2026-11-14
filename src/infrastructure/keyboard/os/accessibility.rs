//! The macOS Accessibility trust check and request. Excluded from the coverage gate.

use std::sync::atomic::{AtomicBool, Ordering};

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::{CFString, CFStringRef};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    #[allow(non_upper_case_globals, reason = "Apple's constant name")]
    static kAXTrustedCheckOptionPrompt: CFStringRef;
}

/// Set once the system prompt has been requested in this app run.
static REQUESTED: AtomicBool = AtomicBool::new(false);

/// Whether this process may post key events. Does not log; callers decide what to say.
pub fn accessibility_trusted() -> bool {
    // SAFETY: `AXIsProcessTrusted` takes no arguments and only reads the process trust state.
    unsafe { AXIsProcessTrusted() }
}

/// Ask macOS to show its Accessibility prompt and add this process to the list. Returns the trust
/// state at the time of the call, which is usually `false` on the first request.
pub fn request_accessibility() -> bool {
    // SAFETY: `kAXTrustedCheckOptionPrompt` is a process-lifetime CFString constant from
    // ApplicationServices. `wrap_under_get_rule` retains it, so the wrapper owns its own reference.
    let key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let options = CFDictionary::from_CFType_pairs(&[(key, CFBoolean::true_value())]);
    // SAFETY: `options` is a live CFDictionary for the duration of this call, and the framework
    // only reads it.
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

/// `request_accessibility`, at most once per app run. Does nothing when already trusted, and
/// returns `false` without a second prompt after the first request.
pub fn request_accessibility_once() -> bool {
    if accessibility_trusted() {
        return true;
    }
    if REQUESTED.swap(true, Ordering::SeqCst) {
        return false;
    }
    request_accessibility()
}
