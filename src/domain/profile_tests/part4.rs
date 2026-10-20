use super::*;

#[test]
fn action_problems_cover_each_variant() {
    assert_eq!(Action::None.problem(1), None);
    assert!(Action::OpenUrl { url: " ".into() }.problem(1).is_some());
    assert_eq!(Action::OpenUrl { url: "u".into() }.problem(1), None);
    assert!(Action::RunCommand {
        command: String::new()
    }
    .problem(1)
    .is_some());
    assert!(Action::OpenApp {
        name: String::new()
    }
    .problem(1)
    .is_some());
    assert_eq!(
        Action::OpenApp {
            name: "Notes".into()
        }
        .problem(1),
        None
    );
    assert!(Action::Hotkey {
        keys: "hyper+a".into()
    }
    .problem(1)
    .is_some());
    assert!(Action::GoToPage { page: 3 }.problem(2).is_some());
    assert_eq!(Action::GoToPage { page: 1 }.problem(2), None);
    assert!(Action::Builtin {
        name: "nope".into()
    }
    .problem(1)
    .is_some());
}

#[test]
fn action_kinds_match_defaults_and_labels() {
    for kind in ActionKind::ALL {
        assert_eq!(Action::default_of(kind).kind(), kind);
        assert!(!kind.label().is_empty());
    }
}

#[test]
fn invalid_foreground_color_is_reported() {
    let mut profile = Profile::default_from_legacy();
    profile.set_key(
        0,
        2,
        KeyConfig {
            fg: "white".into(),
            ..KeyConfig::default()
        },
    );
    assert_eq!(profile.problems().len(), 1);
}

#[test]
fn empty_image_path_is_reported() {
    let mut profile = Profile::default_from_legacy();
    profile.set_key(
        0,
        2,
        KeyConfig {
            image: Some(PathBuf::new()),
            ..KeyConfig::default()
        },
    );
    let problems = profile.problems();
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("image is empty"));
}

#[test]
fn non_numeric_key_is_rejected() {
    let text = "[[pages]]\nname = \"A\"\n\n[pages.keys.x]\nlabel = \"Hi\"\n";
    assert!(toml::from_str::<Profile>(text).is_err());
}
