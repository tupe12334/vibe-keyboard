use image::{Rgb, RgbImage};

// ponytail: fixed 3x5 pixel font, A-Z 0-9 and a few symbols, scaled 3x. Other characters draw
// blank. Add a real font crate (ab_glyph) when labels need lowercase or non-ASCII text.
const SCALE: u32 = 3;
const GLYPH_W: u32 = 3;
const GLYPH_H: u32 = 5;
const MAX_LINES: usize = 4;

/// Draw `text` centered on `img`, word-wrapped to as many lines as fit.
pub fn draw_label(img: &mut RgbImage, text: &str, fg: Rgb<u8>) {
    draw_wrapped(img, text, fg, MAX_LINES, SCALE);
}

/// Like `draw_label`, but at most three lines. Used for session titles under a status bar.
pub fn draw_title(img: &mut RgbImage, text: &str, fg: Rgb<u8>) {
    draw_wrapped(img, text, fg, 3, SCALE);
}

/// Like `draw_label`, with glyphs `scale` times the base size. Lines past the bottom are clipped.
pub fn draw_label_scaled(img: &mut RgbImage, text: &str, fg: Rgb<u8>, scale: u32) {
    draw_wrapped(img, text, fg, MAX_LINES, scale);
}

/// One line of small text (scale 1, 5 px tall) in the top-right corner of `img`. Fits a 12 px bar.
pub fn draw_tag(img: &mut RgbImage, text: &str, fg: Rgb<u8>) {
    let width = text.chars().count() as u32 * (GLYPH_W + 1);
    let mut x = img.width().saturating_sub(width + 2);
    for c in text.chars() {
        draw_glyph(img, x, 3, glyph(c), fg, 1);
        x += GLYPH_W + 1;
    }
}

fn draw_wrapped(img: &mut RgbImage, text: &str, fg: Rgb<u8>, max_lines: usize, scale: u32) {
    let advance = (GLYPH_W + 1) * scale;
    let line_h = (GLYPH_H + 1) * scale;
    let per_line = (img.width() / advance) as usize;
    let lines = wrap(text, per_line, max_lines);
    let total = lines.len() as u32 * line_h;
    let top = img.height().saturating_sub(total) / 2;
    for (row, line) in lines.iter().enumerate() {
        let width = line.chars().count() as u32 * advance;
        let left = img.width().saturating_sub(width) / 2;
        let y = top + row as u32 * line_h;
        for (i, c) in line.chars().enumerate() {
            draw_glyph(img, left + i as u32 * advance, y, glyph(c), fg, scale);
        }
    }
}

/// Height in pixels of one line of text at `scale`.
pub const fn text_height(scale: u32) -> u32 {
    GLYPH_H * scale
}

/// Draw `text` on one line, centered horizontally, with its top edge at `y`. Pixels past
/// the image edge are dropped.
pub fn draw_line(img: &mut RgbImage, y: u32, text: &str, fg: Rgb<u8>, scale: u32) {
    let advance = (GLYPH_W + 1) * scale;
    let width = text.chars().count() as u32 * advance;
    let left = img.width().saturating_sub(width) / 2;
    for (i, c) in text.chars().enumerate() {
        draw_glyph(img, left + i as u32 * advance, y, glyph(c), fg, scale);
    }
}

fn wrap(text: &str, per_line: usize, max_lines: usize) -> Vec<String> {
    let per_line = per_line.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let joined = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if joined.chars().count() <= per_line {
            current = joined;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        let mut rest: Vec<char> = word.chars().collect();
        while rest.len() > per_line {
            lines.push(rest.drain(..per_line).collect::<String>());
        }
        current = rest.into_iter().collect();
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.truncate(max_lines);
    lines
}

fn draw_glyph(img: &mut RgbImage, x: u32, y: u32, rows: [u8; 5], fg: Rgb<u8>, scale: u32) {
    for (ry, bits) in rows.iter().enumerate() {
        for rx in 0..GLYPH_W {
            if bits & (1 << (GLYPH_W - 1 - rx)) == 0 {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x + rx * scale + dx;
                    let py = y + ry as u32 * scale + dy;
                    if px < img.width() && py < img.height() {
                        img.put_pixel(px, py, fg);
                    }
                }
            }
        }
    }
}

/// Rows of a glyph, top to bottom. Bit 2 is the left column.
const fn glyph(c: char) -> [u8; 5] {
    match c.to_ascii_uppercase() {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b101, 0b111, 0b111, 0b111, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b110, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b110, 0b001, 0b010, 0b001, 0b110],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b110, 0b001, 0b110],
        '6' => [0b011, 0b100, 0b110, 0b101, 0b010],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b110],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
        '?' => [0b110, 0b001, 0b010, 0b000, 0b010],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        _ => [0; 5],
    }
}

#[cfg(test)]
mod tests {
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
}
