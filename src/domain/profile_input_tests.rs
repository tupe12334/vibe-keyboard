use super::*;
use std::time::Duration;

#[test]
fn push_to_talk_and_hold_round_trip_through_toml() {
    let text = r#"
[input]
hold_ms = 250

[[pages]]
name = "Main"

[pages.keys."3"]
label = "Dictate"
action = { type = "builtin", name = "terminal" }
hold = { type = "push_to_talk" }
"#;
    let profile = toml::from_str::<Profile>(text).unwrap_or_default();
    assert_eq!(profile.hold_threshold(), Duration::from_millis(250));
    assert_eq!(
        profile.key_at(0, 3).and_then(|config| config.hold.clone()),
        Some(Action::PushToTalk { key: None })
    );
    assert!(profile.problems().is_empty());
    let written = toml::to_string(&profile).unwrap_or_default();
    assert_eq!(toml::from_str::<Profile>(&written), Ok(profile));
}

#[test]
fn missing_input_table_means_default_hold() {
    let profile = toml::from_str::<Profile>("[input]\n").unwrap_or_default();
    assert_eq!(profile.hold_threshold(), Duration::from_millis(400));
    assert!(profile.input.is_default());
}

#[test]
fn existing_profiles_serialize_without_new_fields() {
    let text = toml::to_string(&Profile::default_from_legacy()).unwrap_or_default();
    assert!(!text.contains("hold"));
    assert!(!text.contains("[input]"));
}

#[test]
fn push_to_talk_problems_flag_bad_keys() {
    let mut profile = Profile::default_from_legacy();
    profile.set_key(
        0,
        5,
        KeyConfig {
            action: Action::PushToTalk {
                key: Some("hyper+k".into()),
            },
            hold: Some(Action::PushToTalk {
                key: Some("é".into()),
            }),
            ..KeyConfig::default()
        },
    );
    assert_eq!(profile.problems().len(), 2);
    assert!(Action::PushToTalk { key: None }.problem(1).is_none());
}
