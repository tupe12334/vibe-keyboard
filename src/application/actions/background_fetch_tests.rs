use super::*;

fn project() -> CentyProject {
    CentyProject {
        name: "p".into(),
        org: "o".into(),
        path: None,
        url: "https://app.centy.io/o/p".into(),
    }
}

#[test]
fn no_projects_keeps_the_current_page() {
    assert!(next_screen(Fetched::Projects(vec![])).is_none());
}

#[test]
fn projects_open_the_project_list_on_its_first_page() {
    assert!(matches!(
        next_screen(Fetched::Projects(vec![project()])),
        Some(Screen::CentyProjectList { page: 0, .. })
    ));
}

#[test]
fn sessions_always_open_the_sessions_page() {
    assert!(matches!(
        next_screen(Fetched::Sessions(vec![])),
        Some(Screen::ClaudeSessions { page: 0, .. })
    ));
}
