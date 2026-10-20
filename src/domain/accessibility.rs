//! Push-to-talk posts keys, which needs the Accessibility permission. Pure: the caller reads the OS.

use super::profile::{Action, Profile};

/// Shown in the tray and the editor. The tray item opens the Accessibility settings.
pub const PUSH_TO_TALK_WARNING: &str = "Push-to-talk needs Accessibility — click to open settings";

/// The warning to show, or None. Warns only when the layout uses push-to-talk and this
/// process may not post key events.
pub const fn push_to_talk_warning(trusted: bool, uses_push_to_talk: bool) -> Option<&'static str> {
    if trusted || !uses_push_to_talk {
        None
    } else {
        Some(PUSH_TO_TALK_WARNING)
    }
}

/// Whether any key of `profile` runs push-to-talk, as a tap or as a hold.
pub fn uses_push_to_talk(profile: &Profile) -> bool {
    profile
        .pages
        .iter()
        .flat_map(|page| page.keys.values())
        .any(|key| {
            matches!(key.action, Action::PushToTalk { .. })
                || matches!(key.hold, Some(Action::PushToTalk { .. }))
        })
}

#[cfg(test)]
#[path = "accessibility_tests.rs"]
mod tests;
