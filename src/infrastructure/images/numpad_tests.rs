use super::*;

#[test]
fn generate_numpad_digit_image_valid_size() {
    for d in 1u8..=9 {
        let img = generate_numpad_digit_image(d);
        assert_eq!(img.width(), 100);
        assert_eq!(img.height(), 100);
    }
}

#[test]
fn generate_numpad_digit_image_has_foreground_pixels() {
    // digit 8 lights all segments — must have non-bg pixels
    let img = generate_numpad_digit_image(8);
    let fg = Rgb([220u8, 220, 220]);
    assert!(img.pixels().any(|p| *p == fg));
}

#[test]
fn generate_numpad_clear_image_valid_size() {
    let img = generate_numpad_clear_image();
    assert_eq!(img.width(), 100);
    assert_eq!(img.height(), 100);
}

#[test]
fn generate_numpad_backspace_image_valid_size() {
    let img = generate_numpad_backspace_image();
    assert_eq!(img.width(), 100);
    assert_eq!(img.height(), 100);
}

#[test]
fn generate_numpad_entry_image_valid_size() {
    let img = generate_numpad_entry_image();
    assert_eq!(img.width(), 100);
    assert_eq!(img.height(), 100);
}

#[test]
fn segments_digit_1_only_right_verticals() {
    let (t, tl, tr, m, bl, br, b) = segments(1);
    assert!(!t && !tl && tr && !m && !bl && br && !b);
}

#[test]
fn segments_digit_8_all_on() {
    let (t, tl, tr, m, bl, br, b) = segments(8);
    assert!(t && tl && tr && m && bl && br && b);
}

#[test]
fn segments_unknown_digit_all_off() {
    let (t, tl, tr, m, bl, br, b) = segments(0);
    assert!(!t && !tl && !tr && !m && !bl && !br && !b);
}
