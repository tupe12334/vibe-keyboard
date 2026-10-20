use super::*;

#[test]
fn side_image_is_portrait_100_by_480() {
    let root = PathBuf::from("/nonexistent");
    let metrics = Metrics::default();
    let inset = SideInset::default();
    assert_eq!(
        side_image(None, &root, &metrics, &inset).dimensions(),
        (100, 480)
    );
    let side = SideConfig {
        label: "Build ok".into(),
        ..SideConfig::default()
    };
    assert_eq!(
        side_image(Some(&side), &root, &metrics, &inset).dimensions(),
        (100, 480)
    );
}

#[test]
fn label_stays_inside_the_safe_area() {
    let root = PathBuf::from("/nonexistent");
    let side = SideConfig {
        label: "LONG LABEL THAT WRAPS ACROSS LINES".into(),
        ..black_on_white(Vec::new())
    };
    let inset = SideInset::default();
    let img = side_image(Some(&side), &root, &Metrics::default(), &inset);
    let safe = safe_rect(&inset);
    let inside = |y: u32| (safe.y..safe.y + safe.h).contains(&y);
    for (_, y, pixel) in img.enumerate_pixels() {
        if pixel.0 == [255, 255, 255] {
            assert!(
                inside(y),
                "foreground pixel at y={y} is outside the safe area"
            );
        }
    }
}

#[test]
fn oversized_side_inset_gives_a_solid_background() {
    let root = PathBuf::from("/nonexistent");
    let insets = [
        SideInset {
            top: SIDE_HEIGHT,
            bottom: 0,
        },
        SideInset {
            top: 400,
            bottom: 400,
        },
        SideInset {
            top: u32::MAX,
            bottom: u32::MAX,
        },
    ];
    for widgets in [Vec::new(), all_widgets()] {
        let side = black_on_white(widgets);
        for inset in insets {
            let img = side_image(Some(&side), &root, &sample(), &inset);
            assert_eq!(img.dimensions(), (SIDE_WIDTH, SIDE_HEIGHT));
            assert!(img.pixels().all(|p| p.0 == [0, 0, 0]), "{inset:?}");
        }
    }
}

#[test]
fn tiny_visible_side_rect_does_not_panic() {
    let root = PathBuf::from("/nonexistent");
    let side = black_on_white(all_widgets());
    for visible in 1..=10 {
        let inset = SideInset {
            top: 0,
            bottom: SIDE_HEIGHT - visible,
        };
        let img = side_image(Some(&side), &root, &sample(), &inset);
        assert_eq!(img.dimensions(), (SIDE_WIDTH, SIDE_HEIGHT));
    }
}
