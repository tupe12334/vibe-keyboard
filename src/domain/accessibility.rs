//! Push-to-talk posts keys, and a key combo runs through System Events. Both need the
//! Accessibility permission. Pure: the caller reads the OS.

use super::profile::{Action, Profile};

/// Shown in the tray and the editor. The tray item opens the Accessibility settings.
pub const PUSH_TO_TALK_WARNING: &str = "Keys need Accessibility — click to open settings";

/// Logged once when a key combo runs without the grant.
pub const HOTKEY_WARNING: &str = "hotkey needs Accessibility";

/// The warning to show, or None. Warns only when the layout sends keys and this
/// process may not post key events.
pub const fn push_to_talk_warning(trusted: bool, needs: bool) -> Option<&'static str> {
    if trusted || !needs {
        None
    } else {
        Some(PUSH_TO_TALK_WARNING)
    }
}

/// The log line for a key combo, or None when the grant is held.
pub const fn hotkey_log(trusted: bool) -> Option<&'static str> {
    if trusted {
        None
    } else {
        Some(HOTKEY_WARNING)
    }
}

/// Whether any key of `profile` sends keystrokes, as a tap or as a hold: push-to-talk
/// (CoreGraphics posts), a key combo (System Events), or the Claude sessions page (its
/// held session keys dictate through push-to-talk).
pub fn needs_accessibility(profile: &Profile) -> bool {
    profile
        .pages
        .iter()
        .flat_map(|page| page.keys.values())
        .any(|key| {
            [Some(&key.action), key.hold.as_ref()]
                .into_iter()
                .flatten()
                .any(sends_keys)
        })
}

fn sends_keys(action: &Action) -> bool {
    match action {
        Action::PushToTalk { .. } | Action::Hotkey { .. } => true,
        // The only builtin that posts keys: dictation on its session pages.
        Action::Builtin { name } => name.as_str() == "claude_sessions",
        _ => false,
    }
}

#[cfg(test)]
#[path = "accessibility_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "accessibility_builtin_tests.rs"]
mod builtin_tests;
