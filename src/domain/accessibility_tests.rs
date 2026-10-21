use super::*;
use crate::domain::profile::{KeyConfig, Page};

fn profile_with(key: KeyConfig) -> Profile {
    let mut page = Page::new("main");
    page.keys.insert(1, key);
    Profile {
        pages: vec![page],
        ..Profile::default()
    }
}

fn push_to_talk() -> Action {
    Action::PushToTalk { key: None }
}

fn hotkey() -> Action {
    Action::Hotkey {
        keys: "cmd+shift+4".into(),
    }
}

#[test]
fn warns_only_when_untrusted_and_keys_need_accessibility() {
    assert_eq!(
        push_to_talk_warning(false, true),
        Some(PUSH_TO_TALK_WARNING)
    );
    assert_eq!(push_to_talk_warning(true, true), None);
    assert_eq!(push_to_talk_warning(false, false), None);
    assert_eq!(push_to_talk_warning(true, false), None);
}

#[test]
fn no_keys_do_not_need_accessibility() {
    assert!(!needs_accessibility(&Profile::default()));
}

#[test]
fn non_keystroke_keys_do_not_need_accessibility() {
    let key = KeyConfig {
        action: Action::GoToPage { page: 0 },
        ..KeyConfig::default()
    };
    assert!(!needs_accessibility(&profile_with(key)));
}

#[test]
fn push_to_talk_tap_and_hold_need_accessibility() {
    let tap = KeyConfig {
        action: push_to_talk(),
        ..KeyConfig::default()
    };
    let hold = KeyConfig {
        hold: Some(push_to_talk()),
        ..KeyConfig::default()
    };
    assert!(needs_accessibility(&profile_with(tap)));
    assert!(needs_accessibility(&profile_with(hold)));
}

#[test]
fn key_combo_tap_and_hold_need_accessibility() {
    let tap = KeyConfig {
        action: hotkey(),
        ..KeyConfig::default()
    };
    let hold = KeyConfig {
        hold: Some(hotkey()),
        ..KeyConfig::default()
    };
    assert!(needs_accessibility(&profile_with(tap)));
    assert!(needs_accessibility(&profile_with(hold)));
}

#[test]
fn hotkey_logs_only_without_the_grant() {
    assert_eq!(hotkey_log(false), Some(HOTKEY_WARNING));
    assert_eq!(hotkey_log(true), None);
}

#[test]
fn hotkey_only_profile_shows_the_tray_item_without_the_grant() {
    let key = KeyConfig {
        action: hotkey(),
        ..KeyConfig::default()
    };
    let profile = profile_with(key);
    assert_eq!(
        push_to_talk_warning(false, needs_accessibility(&profile)),
        Some(PUSH_TO_TALK_WARNING)
    );
    assert_eq!(
        push_to_talk_warning(true, needs_accessibility(&profile)),
        None
    );
}
