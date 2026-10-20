use super::*;

#[test]
fn problems_flag_reserved_keys_colors_and_actions() {
    let mut profile = Profile::default_from_legacy();
    profile.set_key(
        0,
        12,
        KeyConfig {
            bg: "red".into(),
            action: Action::GoToPage { page: 9 },
            ..KeyConfig::default()
        },
    );
    assert_eq!(profile.problems().len(), 3);
}

#[test]
fn color_parsing_round_trips() {
    assert_eq!(parse_color("#1A2B3C"), Some([0x1A, 0x2B, 0x3C]));
    assert_eq!(parse_color("1A2B3C"), None);
    assert_eq!(parse_color("#+A2B3C"), None);
    assert_eq!(format_color([0x1A, 0x2B, 0x3C]), "#1A2B3C");
}

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
