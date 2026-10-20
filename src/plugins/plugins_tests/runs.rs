use super::fake_host::FakeHost;
use crate::domain::actions::CentyProject;
use crate::domain::navigation::Screen;
use crate::plugins::find;

/// Runs plugin `name` on `host` and returns the side effects it asked for.
fn run(name: &str, host: &mut FakeHost) -> Vec<String> {
    // A missing plugin makes no calls, so the call assertions fail.
    find(name).into_iter().for_each(|p| p.run(host));
    host.calls.clone()
}

#[test]
fn app_launchers_make_one_call_each() {
    for (name, call) in [
        ("terminal", "terminal"),
        ("claude", "claude"),
        ("centy_web", "chrome https://app.centy.io"),
        ("log_file", "log"),
        ("vscode_config", "config"),
        ("moadim", "routines"),
    ] {
        assert_eq!(run(name, &mut FakeHost::default()), [call], "{name}");
    }
}

#[test]
fn numpad_shows_an_empty_number_input() {
    let mut host = FakeHost::default();
    assert_eq!(run("numpad", &mut host), ["show"]);
    assert!(matches!(&host.shown[..], [Screen::InputNumber { value }] if value.is_empty()));
}

#[test]
fn centy_projects_with_none_shows_nothing() {
    let mut host = FakeHost::default();
    let calls = run("centy_projects", &mut host);
    assert_eq!(calls, ["loading true", "fetch projects", "loading false"]);
}

#[test]
fn centy_projects_shows_the_list_from_page_zero() {
    let mut host = FakeHost {
        projects: vec![CentyProject {
            name: "vibe".into(),
            org: "o".into(),
            path: None,
            url: "u".into(),
        }],
        ..FakeHost::default()
    };
    let calls = run("centy_projects", &mut host);
    assert_eq!(
        calls,
        ["loading true", "fetch projects", "loading false", "show"]
    );
    assert!(matches!(
        &host.shown[..],
        [Screen::CentyProjectList { projects, page: 0, filter: None }] if projects.len() == 1
    ));
}

#[test]
fn claude_sessions_shows_the_session_list() {
    let mut host = FakeHost::default();
    let calls = run("claude_sessions", &mut host);
    assert_eq!(
        calls,
        ["loading true", "list sessions", "loading false", "show"]
    );
    assert!(matches!(
        &host.shown[..],
        [Screen::ClaudeSessions { sessions, page: 0 }] if sessions.is_empty()
    ));
}
