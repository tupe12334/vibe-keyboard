use super::support::{make_issues, make_projects};
use super::*;

// --- back ---
#[test]
fn back_main_page_wraps_from_zero() {
    let mut nav = NavigationStack::new(0, 3);
    nav.back();
    assert!(matches!(nav.current(), Screen::MainPage { page: 2 }));
}

#[test]
fn back_main_page_decrements() {
    let mut nav = NavigationStack::new(2, 3);
    nav.back();
    assert!(matches!(nav.current(), Screen::MainPage { page: 1 }));
}

#[test]
fn back_project_list_decrements_page() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectList {
        projects: make_projects(15),
        page: 1,
        filter: None,
    });
    nav.back();
    assert!(matches!(
        nav.current(),
        Screen::CentyProjectList { page: 0, .. }
    ));
}

#[test]
fn back_project_list_at_page_zero_pops() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyProjectList {
        projects: make_projects(5),
        page: 0,
        filter: None,
    });
    nav.back();
    assert!(matches!(nav.current(), Screen::MainPage { .. }));
}

#[test]
fn back_issue_list_decrements_page() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueList {
        issues: make_issues(15),
        page: 1,
        project_name: "p".into(),
        org: "org".into(),
        filter: None,
    });
    nav.back();
    assert!(matches!(
        nav.current(),
        Screen::CentyIssueList { page: 0, .. }
    ));
}

#[test]
fn back_issue_list_at_page_zero_pops() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::CentyIssueList {
        issues: make_issues(5),
        page: 0,
        project_name: "p".into(),
        org: "org".into(),
        filter: None,
    });
    nav.back();
    assert!(matches!(nav.current(), Screen::MainPage { .. }));
}
