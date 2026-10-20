//! Detects the frontmost macOS application by bundle id.
//!
//! Uses `lsappinfo`, not `NSWorkspace::frontmostApplication`. `NSWorkspace`
//! returned a stale app when polled from a background thread (checked: it
//! kept the start app after a Finder switch). `lsappinfo` reads live state
//! and needs no Accessibility or Screen Recording permission.
//!
//! This file holds the pure logic. Process spawns live in `frontmost/os.rs`.

// The `#[path]` keeps this working when the example includes this file directly.
#[path = "frontmost/os.rs"]
mod os;

// Public API for the app-aware PR. Unused until a caller exists.
#[allow(
    unused_imports,
    reason = "re-export keeps the module surface stable across platforms"
)]
pub use os::{frontmost_bundle_id, watch};

/// Parses `"CFBundleIdentifier"="com.googlecode.iterm2"` into the bundle id.
fn parse_bundle_id(output: &str) -> Option<String> {
    let value = output
        .trim()
        .strip_prefix("\"CFBundleIdentifier\"=\"")?
        .strip_suffix('"')?;
    (!value.is_empty()).then(|| value.to_string())
}

/// Poll state. Each tick costs one `lsappinfo front` call. The bundle lookup
/// runs only when the ASN changes (or the last lookup failed).
#[derive(Default)]
struct Watcher {
    asn: Option<String>,
    bundle: Option<String>,
    sent: Option<String>,
}

impl Watcher {
    /// Returns the bundle id when it changed since the last returned value.
    /// A None `front` or None lookup is never a change.
    fn tick(
        &mut self,
        front: Option<String>,
        lookup: impl FnOnce(&str) -> Option<String>,
    ) -> Option<String> {
        let asn = front?;
        if self.asn.as_deref() != Some(asn.as_str()) || self.bundle.is_none() {
            self.bundle = lookup(&asn);
            self.asn = Some(asn);
        }
        detect_change(&mut self.sent, self.bundle.clone())
    }
}

/// Returns `current` when it differs from `last`, updating `last`.
/// `None` never counts as a change, so the last known app is kept.
fn detect_change(last: &mut Option<String>, current: Option<String>) -> Option<String> {
    let current = current?;
    if last.as_deref() == Some(current.as_str()) {
        return None;
    }
    *last = Some(current.clone());
    Some(current)
}

#[cfg(test)]
#[path = "frontmost_tests.rs"]
mod tests;
