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

#[test]
fn warns_only_when_untrusted_and_push_to_talk_is_used() {
    assert_eq!(
        push_to_talk_warning(false, true),
        Some(PUSH_TO_TALK_WARNING)
    );
    assert_eq!(push_to_talk_warning(true, true), None);
    assert_eq!(push_to_talk_warning(false, false), None);
    assert_eq!(push_to_talk_warning(true, false), None);
}

#[test]
fn default_profile_does_not_use_push_to_talk() {
    assert!(!uses_push_to_talk(&Profile::default()));
}

#[test]
fn tap_and_hold_both_count_as_push_to_talk() {
    let tap = KeyConfig {
        action: push_to_talk(),
        ..KeyConfig::default()
    };
    let hold = KeyConfig {
        hold: Some(push_to_talk()),
        ..KeyConfig::default()
    };
    assert!(uses_push_to_talk(&profile_with(tap)));
    assert!(uses_push_to_talk(&profile_with(hold)));
}

#[test]
fn other_actions_do_not_count() {
    let key = KeyConfig {
        action: Action::GoToPage { page: 0 },
        ..KeyConfig::default()
    };
    assert!(!uses_push_to_talk(&profile_with(key)));
}
