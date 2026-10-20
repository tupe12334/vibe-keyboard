use super::*;

#[test]
fn delete_keeps_at_least_one_page_and_move_reorders() {
    let mut profile = Profile::default_from_legacy();
    assert!(profile.move_page(0, 1));
    assert_eq!(profile.pages[0].name, "Tools");
    assert!(profile.delete_page(0));
    assert!(!profile.delete_page(0));
}

#[test]
fn edit_methods_update_pages_and_keys() {
    let mut profile = Profile::default();
    profile.ensure_one_page();
    assert_eq!(profile.pages.len(), 1);
    let second = profile.add_page("Two");
    assert_eq!(second, 1);
    profile.set_key(
        1,
        3,
        KeyConfig {
            label: "x".into(),
            ..KeyConfig::default()
        },
    );
    profile.set_key(9, 3, KeyConfig::default());
    assert_eq!(profile.key_at(1, 3).map(|k| k.label.as_str()), Some("x"));
    assert!(profile.action_at(1, 3).is_some());
    assert!(profile.key_at(9, 3).is_none());
    profile.clear_key(1, 3);
    profile.clear_key(9, 3);
    assert!(profile.key_at(1, 3).is_none());
    assert!(!profile.move_page(0, 5));
    assert!(!profile.move_page(5, 0));
    assert!(!profile.delete_page(7));
}
