use super::*;

#[test]
fn toml_round_trip_keeps_every_action() {
    let mut profile = Profile::default_from_legacy();
    let page = profile.add_page("Extra");
    let actions = [
        Action::OpenUrl {
            url: "https://example.com".into(),
        },
        Action::RunCommand {
            command: "echo hi".into(),
        },
        Action::OpenApp {
            name: "Notes".into(),
        },
        Action::Hotkey {
            keys: "cmd+shift+4".into(),
        },
        Action::GoToPage { page: 0 },
    ];
    for (i, action) in actions.into_iter().enumerate() {
        let cfg = KeyConfig {
            label: "x".into(),
            action,
            image: Some(PathBuf::from("/tmp/a.png")),
            ..KeyConfig::default()
        };
        profile.set_key(page, u8::try_from(i + 1).unwrap_or(1), cfg);
    }
    let text = toml::to_string_pretty(&profile).unwrap_or_default();
    let back: Profile = toml::from_str(&text).unwrap_or_default();
    assert_eq!(back, profile, "text was:\n{text}");
}

#[test]
fn missing_fields_take_defaults() {
    let text = "[[pages]]\nname = \"A\"\n\n[pages.keys.2]\nlabel = \"Hi\"\n";
    let profile: Profile = toml::from_str(text).unwrap_or_default();
    let key = profile.key_at(0, 2);
    assert_eq!(key.map(|k| k.bg.as_str()), Some(DEFAULT_BG));
    assert_eq!(key.map(|k| &k.action), Some(&Action::None));
}

#[test]
fn action_parses_each_variant_from_toml() {
    let cases = [
        ("type = \"none\"", Action::None),
        (
            "type = \"open_url\"\nurl = \"u\"",
            Action::OpenUrl { url: "u".into() },
        ),
        (
            "type = \"run_command\"\ncommand = \"c\"",
            Action::RunCommand {
                command: "c".into(),
            },
        ),
        (
            "type = \"open_app\"\nname = \"n\"",
            Action::OpenApp { name: "n".into() },
        ),
        (
            "type = \"hotkey\"\nkeys = \"cmd+a\"",
            Action::Hotkey {
                keys: "cmd+a".into(),
            },
        ),
        (
            "type = \"go_to_page\"\npage = 1",
            Action::GoToPage { page: 1 },
        ),
        (
            "type = \"builtin\"\nname = \"terminal\"",
            Action::Builtin {
                name: "terminal".into(),
            },
        ),
    ];
    for (text, expected) in cases {
        let parsed: Action = toml::from_str(text).unwrap_or_default();
        assert_eq!(parsed, expected, "input: {text}");
    }
}

#[test]
fn legacy_profile_is_valid() {
    assert!(Profile::default_from_legacy().problems().is_empty());
}
