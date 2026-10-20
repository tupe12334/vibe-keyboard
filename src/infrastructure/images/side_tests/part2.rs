use super::*;

#[test]
fn side_inset_moves_the_label_into_its_visible_rect() {
    let root = PathBuf::from("/nonexistent");
    let side = SideConfig {
        label: "HI".into(),
        ..black_on_white(Vec::new())
    };
    let inset = SideInset {
        top: 60,
        bottom: 100,
    };
    let img = side_image(Some(&side), &root, &Metrics::default(), &inset);
    let visible = safe_rect(&inset);
    assert_eq!((visible.y, visible.h), (60, 320));
    // Nothing in the hidden top or bottom band, only background.
    for (x, y, pixel) in img.enumerate_pixels() {
        if !(visible.y..visible.y + visible.h).contains(&y) {
            assert_eq!(pixel.0, [0, 0, 0], "x={x} y={y} is in a hidden band");
        }
    }
    assert!(img.pixels().any(|p| p.0 == [255, 255, 255]));
}

#[test]
fn widget_layout_fits_the_safe_area_for_any_count() {
    let root = PathBuf::from("/nonexistent");
    let inset = SideInset::default();
    let stack = stack_rect(safe_rect(&inset));
    for count in 1..=5 {
        let side = black_on_white(all_widgets().into_iter().cycle().take(count).collect());
        let img = side_image(Some(&side), &root, &sample(), &inset);
        // The widget box is the only place that gets color. Everything else stays black.
        for (x, y, pixel) in img.enumerate_pixels() {
            let in_box = (stack.x..stack.x + stack.w).contains(&x)
                && (stack.y..stack.y + stack.h).contains(&y);
            if !in_box {
                assert_eq!(pixel.0, [0, 0, 0], "x={x} y={y} count={count}");
            }
        }
        assert!(img.pixels().any(|p| p.0 == [255, 255, 255]));
    }
}

#[test]
fn widgets_replace_label_and_image() {
    let root = PathBuf::from("/nonexistent");
    let metrics = sample();
    let plain = SideConfig {
        label: "HELLO".into(),
        ..SideConfig::default()
    };
    let with_widgets = SideConfig {
        widgets: vec![Widget::CpuPercent],
        ..plain.clone()
    };
    let inset = SideInset::default();
    assert_ne!(
        side_image(Some(&plain), &root, &metrics, &inset),
        side_image(Some(&with_widgets), &root, &metrics, &inset)
    );
}

#[test]
fn bar_color_thresholds() {
    assert_eq!(bar_color(0), GREEN);
    assert_eq!(bar_color(59), GREEN);
    assert_eq!(bar_color(60), AMBER);
    assert_eq!(bar_color(84), AMBER);
    assert_eq!(bar_color(85), RED);
    assert_eq!(bar_color(100), RED);
}
