use super::*;

#[test]
fn new_session_fullscreen_round_trips_through_toml() {
    let text = "[claude_sessions]\nnew_session_fullscreen = true\n";
    let profile = toml::from_str::<Profile>(text).unwrap_or_default();
    assert!(profile.claude_sessions.new_session_fullscreen);
    let written = toml::to_string(&profile).unwrap_or_default();
    assert!(written.contains("[claude_sessions]"));
    assert_eq!(toml::from_str::<Profile>(&written), Ok(profile));
}

#[test]
fn default_claude_sessions_table_stays_out_of_files() {
    let profile = Profile::default_from_legacy();
    assert!(!profile.claude_sessions.new_session_fullscreen);
    let written = toml::to_string(&profile).unwrap_or_default();
    assert!(!written.contains("claude_sessions"));
}
