use super::*;

#[test]
fn wrap_splits_long_words_and_limits_lines() {
    assert_eq!(wrap("ab cd", 8, 4), vec!["ab cd".to_string()]);
    assert_eq!(wrap("abcdefghij", 4, 4), vec!["abcd", "efgh", "ij"]);
    assert_eq!(wrap("a b c d e", 1, 2), vec!["a", "b"]);
}

#[test]
fn draw_label_paints_foreground_pixels() {
    let mut img = RgbImage::from_pixel(100, 100, Rgb([0, 0, 0]));
    draw_label(&mut img, "HI", Rgb([255, 255, 255]));
    assert!(img.pixels().any(|p| p.0 == [255, 255, 255]));
}

#[test]
fn draw_line_is_centered_and_clipped_at_the_edge() {
    let mut img = RgbImage::from_pixel(60, 20, Rgb([0, 0, 0]));
    // Wider than the image: must not panic, and must paint nothing past the edge.
    draw_line(&mut img, 0, "100%", Rgb([255, 255, 255]), 5);
    assert!(img.pixels().any(|p| p.0 == [255, 255, 255]));
}

#[test]
fn scaled_label_uses_the_larger_glyphs() {
    let mut img = RgbImage::from_pixel(100, 100, Rgb([0, 0, 0]));
    draw_label_scaled(&mut img, "N/A", Rgb([255, 255, 255]), 5);
    // A 5x glyph is 15 pixels wide per column; the top-left block must be lit.
    let lit = img.pixels().filter(|p| p.0 == [255, 255, 255]).count();
    assert!(lit > 25 * 5, "only {lit} lit pixels");
}
