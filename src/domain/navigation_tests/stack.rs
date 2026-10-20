use super::support::{current_issue_numbers, current_project_names, make_project};
use super::*;

#[test]
fn home_page_stays_on_sub_screen() {
    let mut nav = NavigationStack::new(1, 3);
    nav.push(Screen::InputNumber {
        value: String::new(),
    });
    assert_eq!(nav.home_page(), 1);
    assert!(nav.current().side().is_none());
}

#[test]
fn go_to_page_and_set_main_pages_clamp() {
    let mut nav = NavigationStack::new(0, 3);
    nav.go_to_page(2);
    assert!(matches!(nav.current(), Screen::MainPage { page: 2 }));
    nav.go_to_page(9);
    assert!(matches!(nav.current(), Screen::MainPage { page: 2 }));
    nav.set_main_pages(2);
    assert!(matches!(nav.current(), Screen::MainPage { page: 1 }));
    nav.set_main_pages(0);
    assert!(matches!(nav.current(), Screen::MainPage { page: 0 }));
    assert_eq!(nav.main_page(), Some(0));
    nav.push(Screen::InputNumber {
        value: String::new(),
    });
    assert_eq!(nav.main_page(), None);
}

// --- new ---
#[test]
fn new_clamps_initial_page() {
    let nav = NavigationStack::new(5, 3);
    assert!(matches!(nav.current(), Screen::MainPage { page: 2 }));
}

#[test]
fn new_respects_initial_page_within_bounds() {
    let nav = NavigationStack::new(1, 3);
    assert!(matches!(nav.current(), Screen::MainPage { page: 1 }));
}

// --- push / current ---
#[test]
fn push_and_current() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectActions {
        project: make_project(),
    });
    assert!(matches!(nav.current(), Screen::CentyProjectActions { .. }));
}

// --- can_back / can_out / can_forward ---
#[test]
fn can_back_always_true() {
    assert!(NavigationStack::can_back());
}

#[test]
fn can_out_false_at_root() {
    let nav = NavigationStack::new(0, 2);
    assert!(!nav.can_out());
}

#[test]
fn can_out_true_with_depth() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectActions {
        project: make_project(),
    });
    assert!(nav.can_out());
}

#[test]
#[should_panic(expected = "expected CentyProjectList")]
fn current_project_names_panics_on_wrong_screen() {
    let nav = NavigationStack::new(0, 1);
    current_project_names(&nav);
}

#[test]
#[should_panic(expected = "expected CentyIssueList")]
fn current_issue_numbers_panics_on_wrong_screen() {
    let nav = NavigationStack::new(0, 1);
    current_issue_numbers(&nav);
}
