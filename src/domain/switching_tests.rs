use super::*;

#[test]
fn switches_to_matching_page_and_back() {
    let mut state = AutoPage::default();
    assert_eq!(state.on_frontmost(true, 0, Some(2)), Some(2));
    assert_eq!(state.on_frontmost(true, 2, None), Some(0));
}

#[test]
fn manual_paging_is_not_undone_when_no_app_matches() {
    let mut state = AutoPage::default();
    assert_eq!(state.on_frontmost(true, 0, None), None);
    state.on_manual_page();
    assert_eq!(state.on_frontmost(true, 1, None), None);
}

#[test]
fn manual_page_after_auto_switch_becomes_the_base() {
    let mut state = AutoPage::default();
    assert_eq!(state.on_frontmost(true, 0, Some(2)), Some(2));
    state.on_manual_page();
    assert_eq!(state.on_frontmost(true, 2, None), None);
}

#[test]
fn clamp_pulls_a_stale_manual_page_into_range() {
    let mut state = AutoPage::default();
    assert_eq!(state.on_frontmost(true, 4, Some(0)), Some(0));
    state.clamp(2);
    assert_eq!(state.on_frontmost(true, 0, None), Some(1));
}

#[test]
fn sub_screen_blocks_switching() {
    let mut state = AutoPage::default();
    assert_eq!(state.on_frontmost(false, 0, Some(2)), None);
    assert_eq!(state.on_frontmost(true, 0, Some(2)), Some(2));
    assert_eq!(state.on_frontmost(false, 2, None), None);
    assert_eq!(state.on_frontmost(true, 2, None), Some(0));
}

#[test]
fn switching_between_matching_apps_keeps_the_original_base() {
    let mut state = AutoPage::default();
    assert_eq!(state.on_frontmost(true, 0, Some(2)), Some(2));
    assert_eq!(state.on_frontmost(true, 2, Some(3)), Some(3));
    assert_eq!(state.on_frontmost(true, 3, None), Some(0));
}
