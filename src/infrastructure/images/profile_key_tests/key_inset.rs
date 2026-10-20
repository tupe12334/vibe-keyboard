use super::*;

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
