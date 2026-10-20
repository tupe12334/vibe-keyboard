use super::support::{
    current_issue_numbers, current_project_names, make_named_project, make_numbered_issue,
};
use super::*;

#[test]
fn toggle_sort_projects_ascending_then_descending() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::CentyProjectList {
        projects: vec![
            make_named_project("alpha"),
            make_named_project("beta"),
            make_named_project("gamma"),
        ],
        page: 0,
        filter: None,
    });
    // Initially ascending: toggle should sort descending
    nav.toggle_sort();
    assert_eq!(current_project_names(&nav), vec!["gamma", "beta", "alpha"]);
    // Toggle again: should restore ascending
    nav.toggle_sort();
    assert_eq!(current_project_names(&nav), vec!["alpha", "beta", "gamma"]);
}

#[test]
fn toggle_sort_issues_desc_then_asc() {
    let mut nav = NavigationStack::new(0, 1);
    nav.push(Screen::CentyIssueList {
        issues: vec![
            make_numbered_issue(3),
            make_numbered_issue(2),
            make_numbered_issue(1),
        ],
        page: 0,
        project_name: "p".into(),
        org: "org".into(),
        filter: None,
    });
    // Initially descending: toggle should sort ascending
    nav.toggle_sort();
    assert_eq!(current_issue_numbers(&nav), vec![1, 2, 3]);
    // Toggle again: should restore descending
    nav.toggle_sort();
    assert_eq!(current_issue_numbers(&nav), vec![3, 2, 1]);
}

#[test]
fn toggle_sort_noop_on_main_page() {
    let mut nav = NavigationStack::new(0, 2);
    nav.toggle_sort();
    assert!(matches!(nav.current(), Screen::MainPage { page: 0 }));
}
