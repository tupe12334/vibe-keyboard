use super::*;

#[test]
fn side_round_trip_and_absent_side_is_byte_stable() {
    let plain = Profile::default_from_legacy();
    let plain_text = toml::to_string_pretty(&plain).unwrap_or_default();
    assert!(!plain_text.contains("side"));

    let mut profile = plain.clone();
    profile.set_side(
        0,
        Some(SideConfig {
            label: "Build ok".into(),
            bg: "#102030".into(),
            fg: "#FFFFFF".into(),
            image: Some(PathBuf::from("icon:back")),
            widgets: Vec::new(),
            bar_style: None,
        }),
    );
    let text = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(text.contains("[pages.side]"), "text was:\n{text}");
    let back: Profile = toml::from_str(&text).unwrap_or_default();
    assert_eq!(back, profile);
    // Clearing the side returns the exact bytes of the plain layout.
    profile.set_side(0, None);
    let cleared = toml::to_string_pretty(&profile).unwrap_or_default();
    assert_eq!(cleared, plain_text);
}

#[test]
fn widgets_round_trip_and_empty_list_is_omitted() {
    let mut profile = Profile::default_from_legacy();
    profile.set_side(
        0,
        Some(SideConfig {
            widgets: vec![
                Widget::CpuBar,
                Widget::CpuPercent,
                Widget::Temperature,
                Widget::DiskFree,
                Widget::Text {
                    value: "BUILD".into(),
                },
            ],
            ..SideConfig::default()
        }),
    );
    let text = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(text.contains("type = \"cpu_bar\""), "text was:\n{text}");
    assert!(text.contains("type = \"text\""), "text was:\n{text}");
    let back: Profile = toml::from_str(&text).unwrap_or_default();
    assert_eq!(back, profile);
    // A side with no widgets writes no `widgets` key, so old profiles stay byte-identical.
    profile.set_side(0, Some(SideConfig::default()));
    let plain = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(!plain.contains("widgets"), "text was:\n{plain}");
}

#[test]
fn apps_field_is_omitted_when_empty() {
    let text = toml::to_string_pretty(&Profile::default_from_legacy()).unwrap_or_default();
    assert!(!text.contains("apps"));
    let mut profile = Profile::default_from_legacy();
    profile.pages[0].apps = vec!["com.google.Chrome".into()];
    let with = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(with.contains("apps = [\"com.google.Chrome\"]"));
    let back: Profile = toml::from_str(&with).unwrap_or_default();
    assert_eq!(back, profile);
}
