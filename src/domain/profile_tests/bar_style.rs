use super::*;

fn with_style(style: BarStyle) -> Profile {
    let mut profile = Profile::default_from_legacy();
    let side = SideConfig {
        widgets: vec![Widget::CpuBar],
        bar_style: Some(style),
        ..SideConfig::default()
    };
    profile.set_side(0, Some(side));
    profile
}

#[test]
fn a_valid_style_has_no_problems() {
    let style = BarStyle {
        fill: Some("#FF8800".into()),
        track: Some("#101010".into()),
        thickness: Some(BAR_THICKNESS_MIN),
        show_value: Some(false),
        label_color: Some("#FFFFFF".into()),
    };
    assert!(style.problems().is_empty());
    assert!(with_style(style).problems().is_empty());
}

#[test]
fn bad_colors_and_thickness_are_reported_with_the_page() {
    let style = BarStyle {
        fill: Some("orange".into()),
        track: Some("#GGGGGG".into()),
        thickness: Some(BAR_THICKNESS_MAX + 1),
        label_color: Some("#FFF".into()),
        ..BarStyle::default()
    };
    let problems = with_style(style).problems();
    assert_eq!(problems.len(), 4, "{problems:?}");
    assert!(problems
        .iter()
        .any(|p| p.contains("side: bar fill must be #RRGGBB")));
    assert!(problems
        .iter()
        .any(|p| p.contains("bar track must be #RRGGBB")));
    assert!(problems
        .iter()
        .any(|p| p.contains("bar label color must be #RRGGBB")));
    assert!(problems
        .iter()
        .any(|p| p.contains("bar thickness must be 10 to 60 px")));
}

#[test]
fn a_profile_without_bar_style_writes_no_bar_style_text() {
    let mut profile = Profile::default_from_legacy();
    let side = SideConfig {
        widgets: vec![Widget::CpuBar],
        ..SideConfig::default()
    };
    profile.set_side(0, Some(side));
    let text = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(!text.contains("bar_style"), "text was:\n{text}");
}

#[test]
fn a_style_round_trips_through_toml() {
    let profile = with_style(BarStyle {
        fill: Some("#2EA043".into()),
        thickness: Some(24),
        show_value: Some(false),
        ..BarStyle::default()
    });
    let text = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(text.contains("[pages.side.bar_style]"), "text was:\n{text}");
    let back: Profile = toml::from_str(&text).unwrap_or_default();
    assert_eq!(back, profile, "text was:\n{text}");
}

#[test]
fn the_real_layout_writes_no_bar_style() {
    let profile: Profile =
        toml::from_str(include_str!("../../../tests/fixtures/real_profile.toml"))
            .unwrap_or_default();
    assert!(!profile.pages.is_empty(), "the fixture must parse");
    let text = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(!text.contains("bar_style"), "text was:\n{text}");
}

#[test]
fn the_memory_widgets_parse_from_toml() {
    let text = "widgets = [{ type = \"mem_bar\" }, { type = \"cpu_mem_bar\" }]";
    let side: SideConfig = toml::from_str(text).unwrap_or_default();
    assert_eq!(side.widgets, vec![Widget::MemBar, Widget::CpuMemBar]);
}
