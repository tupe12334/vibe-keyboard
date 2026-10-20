use super::*;

#[test]
fn empty_key_is_black_and_app_sources_fall_back() {
    let root = PathBuf::from("/nonexistent");
    let inset = KeyInset::default();
    let blank = key_image(5, None, &root, &inset);
    assert!(blank.pixels().all(|p| p.0 == BLANK));
    // An app that is not installed falls back to the label.
    let app = KeyConfig {
        image: Some(PathBuf::from("app:com.example.NotInstalled")),
        label: "Hi".into(),
        ..KeyConfig::default()
    };
    assert_eq!(
        key_image(5, Some(&app), &root, &inset),
        inset_key(
            &label_tile("Hi", FALLBACK_BG, FALLBACK_FG),
            &inset,
            FALLBACK_BG
        )
    );
}
