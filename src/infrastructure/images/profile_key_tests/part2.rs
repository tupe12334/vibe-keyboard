use super::*;

#[test]
fn oversized_key_inset_gives_a_solid_key_color() {
    let root = PathBuf::from("/nonexistent");
    let config = KeyConfig {
        label: "HI".into(),
        bg: "#000000".into(),
        fg: "#FFFFFF".into(),
        ..KeyConfig::default()
    };
    // Insets at or past the key size leave no visible rect. Each must give a full
    // deck-size image in the key color, with no panic.
    let insets = [
        KeyInset {
            top: DECK_IMAGE_SIZE,
            bottom: 0,
            left: 0,
            right: 0,
        },
        KeyInset {
            top: 200,
            bottom: 200,
            left: 200,
            right: 200,
        },
        KeyInset {
            top: u32::MAX,
            bottom: u32::MAX,
            left: u32::MAX,
            right: u32::MAX,
        },
        KeyInset {
            top: 0,
            bottom: 0,
            left: DECK_IMAGE_SIZE,
            right: 1,
        },
    ];
    for inset in insets {
        let img = key_image(5, Some(&config), &root, &inset);
        assert_eq!(img.dimensions(), (DECK_IMAGE_SIZE, DECK_IMAGE_SIZE));
        assert!(img.pixels().all(|p| p.0 == [0, 0, 0]), "{inset:?}");
    }
}

#[test]
fn label_session_and_calibration_tiles_respect_the_inset() {
    let inset = KeyInset {
        top: 20,
        bottom: 0,
        left: 0,
        right: 0,
    };
    let label = label_image("HI", &inset);
    assert_eq!(label.dimensions(), (DECK_IMAGE_SIZE, DECK_IMAGE_SIZE));
    assert!((0..20).all(|y| label.get_pixel(0, y).0 == FALLBACK_BG));
    let session = session_image("X", SessionStatus::Active, "1m", &inset);
    assert_eq!(session.dimensions(), (DECK_IMAGE_SIZE, DECK_IMAGE_SIZE));
    assert!((0..20).all(|y| session.get_pixel(0, y).0 == FALLBACK_BG));
    // Calibration: the red outline starts on the inset edge, and the hidden band stays black.
    let tile = calibration_image(7, &inset);
    assert_eq!(tile.dimensions(), (DECK_IMAGE_SIZE, DECK_IMAGE_SIZE));
    assert_eq!(tile.get_pixel(40, 20).0, CALIBRATION_LINE);
    assert!((0..20).all(|y| tile.get_pixel(40, y).0 == [0, 0, 0]));
    assert!(tile.pixels().any(|p| p.0 == [255, 255, 255]));
}

#[test]
fn oversized_calibration_inset_does_not_panic() {
    let inset = KeyInset {
        top: u32::MAX,
        bottom: u32::MAX,
        left: u32::MAX,
        right: u32::MAX,
    };
    let tile = calibration_image(3, &inset);
    assert_eq!(tile.dimensions(), (DECK_IMAGE_SIZE, DECK_IMAGE_SIZE));
}
