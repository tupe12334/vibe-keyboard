use super::*;
use crate::domain::profile::Page;

#[test]
fn hold_time_is_a_save_problem_outside_its_bounds() {
    let mut profile = Profile::default_from_legacy();
    for (ms, ok) in [(49, false), (50, true), (5000, true), (5001, false)] {
        profile.input.hold_ms = ms;
        let found = save_problems(&profile)
            .iter()
            .any(|p| p.starts_with("hold time"));
        assert_eq!(found, !ok, "hold_ms {ms}");
    }
}

#[test]
fn pages_are_named_by_position_and_never_drop_below_the_minimum() {
    let mut profile = Profile::default();
    assert_eq!(next_page_name(&profile), "Page 1");
    let index = add_page(&mut profile);
    assert_eq!(profile.pages[index].name, "Page 1");
    let second = add_page(&mut profile);
    assert_eq!(profile.pages[second].name, "Page 2");
    assert!(can_delete_page(&profile));
    profile.pages.truncate(MIN_PAGES);
    assert!(!can_delete_page(&profile));
}

#[test]
fn save_refuses_an_empty_page_list_and_blank_names() {
    let empty = Profile {
        pages: vec![],
        ..Profile::default()
    };
    assert!(save_problems(&empty)
        .iter()
        .any(|p| p.contains("at least one page")));
    let blank = Profile {
        pages: vec![Page::new("  ")],
        ..Profile::default()
    };
    assert!(save_problems(&blank)
        .iter()
        .any(|p| p == "page 1: name is empty"));
}

#[test]
fn nav_keys_are_the_reserved_three() {
    assert_eq!(nav_keys(), vec![11, 12, 13]);
}
