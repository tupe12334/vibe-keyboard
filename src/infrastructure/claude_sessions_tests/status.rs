use super::*;

#[test]
fn label_is_home_tilde_or_last_component() {
    assert_eq!(label_for("/Users/me", "/Users/me"), "~");
    assert_eq!(label_for("/Users/me/dev/app", "/Users/me"), "app");
    assert_eq!(label_for("/Users/me/dev/app/", "/Users/me"), "app");
    assert_eq!(label_for("/", "/Users/me"), "/");
}

#[test]
fn glyph_is_read_and_removed_with_the_suffix() {
    assert_eq!(
        parse_title("✳ Centy availability (claude)"),
        (Some(Glyph::Waiting), "Centy availability".to_string())
    );
    assert_eq!(
        parse_title("◐ USB devices overview (claude)"),
        (Some(Glyph::Spinner), "USB devices overview".to_string())
    );
    assert_eq!(
        parse_title("⠋ Spinning"),
        (Some(Glyph::Spinner), "Spinning".to_string())
    );
    assert_eq!(parse_title("  zsh  "), (None, "zsh".to_string()));
}

#[test]
fn status_is_active_within_a_minute_or_when_spinning() {
    let quiet = Duration::from_secs(61);
    assert_eq!(status_for(Duration::ZERO, None), SessionStatus::Active);
    assert_eq!(
        status_for(Duration::from_secs(60), None),
        SessionStatus::Active
    );
    assert_eq!(status_for(quiet, None), SessionStatus::Stale);
    assert_eq!(
        status_for(quiet, Some(Glyph::Waiting)),
        SessionStatus::Stale
    );
    assert_eq!(
        status_for(quiet, Some(Glyph::Spinner)),
        SessionStatus::Active
    );
}

#[test]
fn spinner_keeps_active_only_within_ten_minutes() {
    let nine_min = Duration::from_mins(9);
    let eleven_min = Duration::from_mins(11);
    assert_eq!(
        status_for(nine_min, Some(Glyph::Spinner)),
        SessionStatus::Active
    );
    assert_eq!(
        status_for(eleven_min, Some(Glyph::Spinner)),
        SessionStatus::Stale
    );
    assert_eq!(
        status_for(Duration::MAX, Some(Glyph::Spinner)),
        SessionStatus::Stale
    );
}

#[test]
fn age_label_units() {
    assert_eq!(age_label(None), "");
    assert_eq!(age_label(Some(Duration::from_secs(5))), "NOW");
    assert_eq!(age_label(Some(Duration::from_secs(120))), "2M");
    assert_eq!(age_label(Some(Duration::from_secs(3600))), "1H");
    assert_eq!(age_label(Some(Duration::from_hours(72))), "3D");
}
