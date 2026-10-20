//! Hold action, hold time and app icon edits on the real layout.
#![allow(clippy::expect_used)]

use std::path::PathBuf;

use crate::editor_preserve_support::assert_untouched;
use vibe_keyboard::domain::profile::Action;

#[test]
fn hold_action_edit_keeps_the_key_and_its_tap_action() {
    assert_untouched(
        "hold-action",
        |mut p| {
            p.pages[0].keys.get_mut(&6).expect("key 6").hold = Some(Action::Hotkey {
                keys: "cmd+k".into(),
            });
            p
        },
        &[&["pages", "0", "keys", "6", "hold"]],
    );
}

#[test]
fn hold_time_edit_keeps_the_rest_of_the_layout() {
    assert_untouched(
        "hold-ms",
        |mut p| {
            p.input.hold_ms = 600;
            p
        },
        &[&["input", "hold_ms"]],
    );
}

#[test]
fn app_icon_edit_keeps_the_key_label_and_action() {
    assert_untouched(
        "app-icon",
        |mut p| {
            p.pages[0].keys.get_mut(&14).expect("key 14").image =
                Some(PathBuf::from("app:com.apple.Terminal"));
            p
        },
        &[&["pages", "0", "keys", "14", "image"]],
    );
}
