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
fn key_content_stays_below_the_top_inset() {
    let root = PathBuf::from("/nonexistent");
    let config = KeyConfig {
        label: "HI".into(),
        bg: "#000000".into(),
        fg: "#FFFFFF".into(),
        ..KeyConfig::default()
    };
    let inset = KeyInset {
        top: 20,
        bottom: 0,
        left: 0,
        right: 0,
    };
    let img = key_image(5, Some(&config), &root, &inset);
    // The hidden band keeps the key color. No text pixel may land in it.
    for y in 0..20 {
        for x in 0..DECK_IMAGE_SIZE {
            assert_eq!(img.get_pixel(x, y).0, [0, 0, 0], "x={x} y={y}");
        }
    }
    assert!(img.pixels().any(|p| p.0 == [255, 255, 255]));
}
