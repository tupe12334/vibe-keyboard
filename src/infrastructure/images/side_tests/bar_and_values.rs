use super::*;

#[test]
fn bar_fills_from_the_bottom_up() {
    let root = PathBuf::from("/nonexistent");
    let side = black_on_white(vec![Widget::CpuBar]);
    let at = |percent: u8| {
        let metrics = Metrics {
            cpu_percent: percent,
            ..Metrics::default()
        };
        side_image(Some(&side), &root, &metrics, &SideInset::default())
    };
    let stack = stack_rect(safe_rect(&SideInset::default()));
    let (top, bottom) = bar_span(stack.h, true);
    let x = BAR_CENTER;
    let y0 = stack.y;
    // Half full: the lower half is green, the top of the bar is still track.
    let half = at(50);
    assert_eq!(half.get_pixel(x, y0 + bottom - 1).0, GREEN);
    assert_eq!(half.get_pixel(x, y0 + top).0, TRACK);
    // Full: the top row of the bar is red.
    let full = at(100);
    assert_eq!(full.get_pixel(x, y0 + top).0, RED);
    // Empty: no fill at all.
    let empty = at(0);
    assert_eq!(empty.get_pixel(x, y0 + bottom - 1).0, TRACK);
}

#[test]
fn value_text_formats() {
    assert_eq!(percent(7), "7%");
    assert_eq!(temperature_text(Some(71.4)), "71C");
    assert_eq!(temperature_text(None), "N/A");
    assert_eq!(free_text(None), "N/A");
    assert_eq!(
        free_text(Some(Disk {
            free: 412_000_000_000,
            total: 1
        })),
        "412G"
    );
    assert_eq!(
        free_text(Some(Disk {
            free: 1_200_000_000_000,
            total: 1
        })),
        "1.2T"
    );
}
