//! Key edits on the real layout: label, action, clear and new key.
#![allow(clippy::expect_used)]

use crate::editor_preserve_support::assert_untouched;
use vibe_keyboard::domain::profile::{Action, KeyConfig};

#[test]
fn key_label_edit_keeps_the_other_keys() {
    assert_untouched(
        "key-label",
        |mut p| {
            p.pages[0].keys.get_mut(&1).expect("key 1").label = "Renamed".into();
            p
        },
        &[&["pages", "0", "keys", "1", "label"]],
    );
}

#[test]
fn key_action_edit_keeps_the_other_keys() {
    assert_untouched(
        "key-action",
        |mut p| {
            p.pages[0].keys.get_mut(&2).expect("key 2").action = Action::Hotkey {
                keys: "cmd+shift+4".into(),
            };
            p
        },
        &[&["pages", "0", "keys", "2", "action"]],
    );
}

#[test]
fn clearing_a_key_keeps_the_other_keys() {
    assert_untouched(
        "key-clear",
        |mut p| {
            p.clear_key(0, 9);
            p
        },
        &[&["pages", "0", "keys", "9"]],
    );
}

#[test]
fn adding_a_key_keeps_the_other_keys() {
    assert_untouched(
        "key-add",
        |mut p| {
            p.set_key(
                0,
                10,
                KeyConfig {
                    label: "New".into(),
                    ..KeyConfig::default()
                },
            );
            p
        },
        &[&["pages", "0", "keys", "10"]],
    );
}
