//! Detects the frontmost macOS application by bundle id.
//!
//! Uses `lsappinfo`, not `NSWorkspace::frontmostApplication`. `NSWorkspace`
//! returned a stale app when polled from a background thread (checked: it
//! kept the start app after a Finder switch). `lsappinfo` reads live state
//! and needs no Accessibility or Screen Recording permission.

use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// Bundle id of the frontmost app, e.g. `com.google.Chrome`. None if unknown.
#[cfg(target_os = "macos")]
pub fn frontmost_bundle_id() -> Option<String> {
    let asn = lsappinfo(&["front"])?;
    parse_bundle_id(&lsappinfo(&["info", "-only", "bundleid", &asn])?)
}

#[cfg(target_os = "macos")]
fn lsappinfo(args: &[&str]) -> Option<String> {
    let out = Command::new("lsappinfo").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Parses `"CFBundleIdentifier"="com.googlecode.iterm2"` into the bundle id.
fn parse_bundle_id(output: &str) -> Option<String> {
    let value = output
        .trim()
        .strip_prefix("\"CFBundleIdentifier\"=\"")?
        .strip_suffix('"')?;
    (!value.is_empty()).then(|| value.to_string())
}

/// Bundle id of the frontmost app. Always None off macOS.
#[cfg(not(target_os = "macos"))]
pub const fn frontmost_bundle_id() -> Option<String> {
    None
}

/// Spawn a thread that polls every `interval` and sends the bundle id on each
/// change. The first value is sent immediately.
///
/// The thread notices a dropped receiver only when it next tries to send, so
/// it exits on the first change after the receiver is gone.
pub fn watch(interval: Duration) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut last = None;
        loop {
            if let Some(id) = detect_change(&mut last, frontmost_bundle_id()) {
                if tx.send(id).is_err() {
                    break;
                }
            }
            std::thread::sleep(interval);
        }
    });
    rx
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
    use super::{detect_change, parse_bundle_id};

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
    fn returning_to_previous_app_after_none_is_a_change() {
        let mut last = Some("com.a".into());
        assert_eq!(detect_change(&mut last, None), None);
        assert_eq!(
            detect_change(&mut last, Some("com.b".into())),
            Some("com.b".into())
        );
    }
}
