use super::*;

#[test]
fn prompts_round_trip_and_stay_out_of_files_when_empty() {
    let mut profile = Profile::default_from_legacy();
    assert!(profile.prompts.is_empty(), "first run writes no prompts");
    profile.prompts = vec![Prompt {
        label: "Ask".into(),
        text: "Ask me".into(),
    }];
    let text = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(text.contains("[[prompts]]"));
    let back: Profile = toml::from_str(&text).unwrap_or_default();
    assert_eq!(back, profile);
    assert_eq!(profile.session_prompts(), profile.prompts);

    profile.prompts.clear();
    let text = toml::to_string_pretty(&profile).unwrap_or_default();
    assert!(!text.contains("prompts"));
    let back: Profile = toml::from_str(&text).unwrap_or_default();
    assert_eq!(back, profile);
    assert!(profile.session_prompts().is_empty());
}

#[test]
fn profile_without_prompts_has_no_session_prompts() {
    let old: Profile = toml::from_str("[[pages]]\nname = \"Main\"\n").unwrap_or_default();
    assert!(old.prompts.is_empty());
    assert!(old.session_prompts().is_empty());
}

#[test]
fn prompt_problems_flag_empty_multiline_and_long_text() {
    let prompt = |label: &str, text: &str| Prompt {
        label: label.into(),
        text: text.into(),
    };
    let profile = Profile {
        pages: Vec::new(),
        prompts: vec![
            prompt("", "/a"),
            prompt("B", "  "),
            prompt("C", "line\nbreak"),
            prompt("D", &"x".repeat(PROMPT_MAX_CHARS + 1)),
            prompt("E", "/ok"),
            prompt("F\nG", "/f"),
            prompt(&"L".repeat(PROMPT_LABEL_MAX_CHARS + 1), "/l"),
        ],
        display: DisplaySettings::default(),
    };
    let problems = profile.problems();
    assert_eq!(problems.len(), 6, "{problems:?}");
    assert!(problems[0].contains("label is empty"));
    assert!(problems[1].contains("text is empty"));
    assert!(problems[2].contains("one line"));
    assert!(problems[3].contains("over"));
    assert!(problems[4].contains("label must be one line"));
    assert!(problems[5].contains("label is over"));
}
