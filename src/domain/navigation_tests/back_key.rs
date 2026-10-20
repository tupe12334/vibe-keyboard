use super::support::make_numbered_issue;
use super::*;

#[test]
fn key_11_is_back_on_main_pages_and_out_on_sub_screens() {
    let project = CentyProject {
        name: "p".into(),
        org: "o".into(),
        path: None,
        url: String::new(),
    };
    let screens = [
        (Screen::MainPage { page: 0 }, BackKey::Back),
        (
            Screen::CentyProjectList {
                projects: vec![],
                page: 0,
                filter: None,
            },
            BackKey::Out,
        ),
        (
            Screen::CentyProjectActions {
                project: project.clone(),
            },
            BackKey::Out,
        ),
        (
            Screen::CentyIssueList {
                issues: vec![],
                page: 0,
                project_name: "p".into(),
                org: "o".into(),
                filter: None,
            },
            BackKey::Out,
        ),
        (
            Screen::CentyIssueActions {
                issue: make_numbered_issue(1),
                project_name: "p".into(),
                org: "o".into(),
            },
            BackKey::Out,
        ),
        (
            Screen::InputNumber {
                value: String::new(),
            },
            BackKey::Out,
        ),
        (
            Screen::ClaudeSessions {
                sessions: vec![],
                page: 0,
            },
            BackKey::Out,
        ),
        (Screen::NewSessionChoice, BackKey::Out),
    ];
    for (screen, expected) in screens {
        assert_eq!(back_key_on(&screen), expected);
    }
}
