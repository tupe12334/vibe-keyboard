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
    reason = "public API for the app-aware PR; unused until a caller exists"
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
mod tests {
    use super::{detect_change, parse_bundle_id, Watcher};
    use std::cell::Cell;

    #[test]
    fn parses_lsappinfo_bundle_line() {
        assert_eq!(
            parse_bundle_id("\"CFBundleIdentifier\"=\"com.googlecode.iterm2\"\n"),
            Some("com.googlecode.iterm2".into())
        );
    }

    #[test]
    fn rejects_missing_or_empty_bundle_line() {
        assert_eq!(parse_bundle_id(""), None);
        assert_eq!(parse_bundle_id("[ NULL ]"), None);
        assert_eq!(parse_bundle_id("\"CFBundleIdentifier\"=\"\""), None);
    }

    #[test]
    fn first_value_is_a_change() {
        let mut last = None;
        assert_eq!(
            detect_change(&mut last, Some("com.a".into())),
            Some("com.a".into())
        );
    }

    #[test]
    fn same_value_is_not_a_change() {
        let mut last = Some("com.a".into());
        assert_eq!(detect_change(&mut last, Some("com.a".into())), None);
    }

    #[test]
    fn new_value_is_a_change_and_updates_last() {
        let mut last = Some("com.a".into());
        assert_eq!(
            detect_change(&mut last, Some("com.b".into())),
            Some("com.b".into())
        );
        assert_eq!(last.as_deref(), Some("com.b"));
    }

    #[test]
    fn none_is_ignored_and_keeps_last() {
        let mut last = Some("com.a".into());
        assert_eq!(detect_change(&mut last, None), None);
        assert_eq!(last.as_deref(), Some("com.a"));
    }

    #[test]
    fn new_app_after_none_is_a_change() {
        let mut last = Some("com.a".into());
        assert_eq!(detect_change(&mut last, None), None);
        assert_eq!(
            detect_change(&mut last, Some("com.b".into())),
            Some("com.b".into())
        );
    }

    #[test]
    fn tick_sends_first_bundle_then_nothing_for_same_asn() {
        let lookups = Cell::new(0);
        let lookup = |_: &str| {
            lookups.set(lookups.get() + 1);
            Some("com.a".into())
        };
        let mut w = Watcher::default();
        assert_eq!(w.tick(Some("asn1".into()), lookup), Some("com.a".into()));
        assert_eq!(w.tick(Some("asn1".into()), lookup), None);
        assert_eq!(lookups.get(), 1, "lookup must run once per ASN");
    }

    #[test]
    fn tick_looks_up_again_only_when_asn_changes() {
        let mut w = Watcher::default();
        assert_eq!(
            w.tick(Some("asn1".into()), |_| Some("com.a".into())),
            Some("com.a".into())
        );
        assert_eq!(
            w.tick(Some("asn2".into()), |_| Some("com.b".into())),
            Some("com.b".into())
        );
    }

    #[test]
    fn tick_new_asn_with_same_bundle_is_not_sent_again() {
        let mut w = Watcher::default();
        assert_eq!(
            w.tick(Some("asn1".into()), |_| Some("com.a".into())),
            Some("com.a".into())
        );
        assert_eq!(w.tick(Some("asn2".into()), |_| Some("com.a".into())), None);
    }

    #[test]
    fn tick_none_front_keeps_state_and_skips_lookup() {
        let mut w = Watcher::default();
        let lookup = |_: &str| Some("com.a".into());
        assert_eq!(w.tick(Some("asn1".into()), lookup), Some("com.a".into()));
        assert_eq!(w.tick(None, lookup), None);
        assert_eq!(w.asn.as_deref(), Some("asn1"));
    }

    #[test]
    fn tick_retries_lookup_after_a_failed_one() {
        let mut w = Watcher::default();
        assert_eq!(w.tick(Some("asn1".into()), |_| None), None);
        assert_eq!(
            w.tick(Some("asn1".into()), |_| Some("com.a".into())),
            Some("com.a".into())
        );
    }
}
