use super::*;

#[test]
fn every_key_gives_a_square_bitmap() {
    let profile = Profile::default_from_legacy();
    let root = PathBuf::from("/nonexistent");
    for key in 1..=15u8 {
        let config = profile.key_at(0, key);
        let img = key_image(key, config, &root, &KeyInset::default());
        assert_eq!(
            img.dimensions(),
            (DECK_IMAGE_SIZE, DECK_IMAGE_SIZE),
            "key {key}"
        );
    }
}

#[test]
fn page_images_lists_every_key_once_in_order() {
    let profile = Profile::default_from_legacy();
    let root = PathBuf::from("/nonexistent");
    let keys = &profile.pages[0].keys;
    let inset = KeyInset::default();
    let images = page_images(keys, &root, &inset);
    let numbers: Vec<u8> = images.iter().map(|(key, _)| *key).collect();
    assert_eq!(numbers, (1..=15).collect::<Vec<u8>>());
    for (key, img) in images {
        assert_eq!(
            img,
            key_image(key, keys.get(&key), &root, &inset),
            "key {key}"
        );
    }
}

#[test]
fn nav_keys_do_not_depend_on_the_profile() {
    let root = PathBuf::from("/nonexistent");
    let inset = KeyInset::default();
    let config = KeyConfig {
        label: "x".into(),
        ..KeyConfig::default()
    };
    assert_eq!(
        key_image(11, Some(&config), &root, &inset),
        key_image(11, None, &root, &inset)
    );
    assert_ne!(
        key_image(12, None, &root, &inset),
        key_image(13, None, &root, &inset)
    );
}

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
