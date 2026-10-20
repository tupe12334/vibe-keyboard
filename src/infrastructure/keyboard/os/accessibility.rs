//! The macOS Accessibility trust check. Excluded from the coverage gate.

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

/// Whether this process may post key events. Does not log; callers decide what to say.
pub fn accessibility_trusted() -> bool {
    // SAFETY: `AXIsProcessTrusted` takes no arguments and only reads the process trust state.
    unsafe { AXIsProcessTrusted() }
}
