use super::*;
use crate::domain::profile::{KeyConfig, Page};

fn builtin_profile(name: &str) -> Profile {
    let key = KeyConfig {
        action: Action::Builtin { name: name.into() },
        ..KeyConfig::default()
    };
    let mut page = Page::new("main");
    page.keys.insert(1, key);
    Profile {
        pages: vec![page],
        ..Profile::default()
    }
}

#[test]
fn claude_sessions_builtin_needs_accessibility() {
    assert!(needs_accessibility(&builtin_profile("claude_sessions")));
}

#[test]
fn unrelated_builtin_does_not_need_accessibility() {
    assert!(!needs_accessibility(&builtin_profile("terminal")));
}
