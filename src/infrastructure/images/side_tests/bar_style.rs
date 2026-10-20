use super::*;

use crate::domain::profile::{
    parse_color, BarStyle, BAR_FILL_DEFAULT, BAR_THICKNESS_DEFAULT, BAR_TRACK_DEFAULT,
};

const MAGENTA: [u8; 3] = [255, 0, 255];
const BLUE: [u8; 3] = [0, 0, 255];
const LIME: [u8; 3] = [0, 255, 0];
const BLACK: [u8; 3] = [0, 0, 0];

fn at_percent(style: Option<BarStyle>, percent: u8) -> RgbImage {
    let side = SideConfig {
        bar_style: style,
        ..black_on_white(vec![Widget::CpuBar])
    };
    let metrics = Metrics {
        cpu_percent: percent,
        ..Metrics::default()
    };
    side_image(
        Some(&side),
        &PathBuf::from("/nonexistent"),
        &metrics,
        &SideInset::default(),
    )
}

#[test]
fn defaults_match_the_built_in_look() {
    assert_eq!(parse_color(BAR_TRACK_DEFAULT), Some(TRACK));
    assert_eq!(parse_color(BAR_FILL_DEFAULT), Some(GREEN));
    assert_eq!(BAR_THICKNESS_DEFAULT, BAR_W);
    let look = BarLook::resolve(None, [1, 2, 3]);
    assert_eq!((look.fill, look.track, look.width), (None, TRACK, BAR_W));
    assert_eq!(look.value_color, Some([1, 2, 3]));
    // An empty style renders the same pixels as no style.
    assert_eq!(
        at_percent(None, 50),
        at_percent(Some(BarStyle::default()), 50)
    );
}

#[test]
fn fill_track_and_thickness_come_from_the_style() {
    let style = BarStyle {
        fill: Some("#FF00FF".into()),
        track: Some("#0000FF".into()),
        thickness: Some(20),
        ..BarStyle::default()
    };
    let img = at_percent(Some(style), 50);
    let stack = stack_rect(safe_rect(&SideInset::default()));
    let (top, bottom) = bar_span(stack.h, true);
    let x = stack.x + 50;
    let y0 = stack.y;
    assert_eq!(img.get_pixel(x, y0 + bottom - 1).0, MAGENTA);
    assert_eq!(img.get_pixel(x, y0 + top).0, BLUE);
    // Relative column 31 is outside the 20 px bar, so it keeps the background.
    assert_eq!(img.get_pixel(stack.x + 31, y0 + bottom - 1).0, BLACK);
}

#[test]
fn hiding_the_value_gives_the_bar_the_space() {
    let style = BarStyle {
        show_value: Some(false),
        ..BarStyle::default()
    };
    let img = at_percent(Some(style), 100);
    let stack = stack_rect(safe_rect(&SideInset::default()));
    let (_, bottom) = bar_span(stack.h, false);
    assert!(bottom > bar_span(stack.h, true).1);
    // Full bar, no label: the bottom row of the bar is the threshold red.
    assert_eq!(img.get_pixel(stack.x + 50, stack.y + bottom - 1).0, RED);
}

#[test]
fn label_color_paints_the_percent_text() {
    let style = BarStyle {
        label_color: Some("#00FF00".into()),
        ..BarStyle::default()
    };
    let img = at_percent(Some(style), 40);
    let stack = stack_rect(safe_rect(&SideInset::default()));
    let below_bar = stack.y..stack.y + stack.h;
    let found = below_bar
        .clone()
        .any(|y| (stack.x..stack.x + stack.w).any(|x| img.get_pixel(x, y).0 == LIME));
    assert!(found, "no lime text pixel in the stack");
}
