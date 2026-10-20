use super::tests::page_actions;
use crate::domain::profile::{Action, Profile};

#[test]
fn default_from_legacy_matches_page_actions() {
    let profile = Profile::default_from_legacy();
    let expected_name = |name: &str| match name {
        "Centy" => "centy_projects",
        "Terminal" => "terminal",
        "Claude" => "claude",
        "Centy Web" => "centy_web",
        "NumPad" => "numpad",
        "Log File" => "log_file",
        "VSCode Config" => "vscode_config",
        other => panic!("unmapped legacy name {other}"),
    };
    for page in 0..2 {
        let legacy = page_actions(page);
        assert_eq!(
            profile.pages[page].keys.len(),
            legacy.len(),
            "page {page} key count"
        );
        for (key, button) in legacy {
            let cfg = profile.key_at(page, key);
            assert_eq!(cfg.map(|k| k.label.as_str()), Some(button.title.as_str()));
            assert_eq!(
                cfg.map(|k| k.action.clone()),
                Some(Action::Builtin {
                    name: expected_name(&button.name).into()
                })
            );
        }
    }
}
