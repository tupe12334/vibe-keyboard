use super::content_box::content_bbox;
use super::*;
use crate::domain::profile::inner_rect;

#[test]
#[allow(
    clippy::print_stderr,
    reason = "diagnostic report for the key layout check"
)]
fn key_content_fills_the_visible_rect() {
    use crate::domain::profile::BUILTIN_NAMES;
    use crate::infrastructure::images::ICON_NAMES;

    let root = PathBuf::from("/nonexistent");
    let inset = KeyInset {
        top: 10,
        bottom: 0,
        left: 0,
        right: 0,
    };
    let rect = inner_rect(
        DECK_IMAGE_SIZE,
        DECK_IMAGE_SIZE,
        inset.top,
        inset.bottom,
        inset.left,
        inset.right,
    );
    let bg_hex = "#1E3C78";
    let bg = parse_color(bg_hex).unwrap_or([0; 3]);
    let mut icons: Vec<(String, KeyConfig)> = Vec::new();
    for name in BUILTIN_NAMES {
        // The session list tile is drawn by its page, not as a key icon.
        if name == "claude_sessions" {
            continue;
        }
        icons.push((
            name.to_string(),
            KeyConfig {
                bg: bg_hex.into(),
                action: Action::Builtin { name: name.into() },
                ..KeyConfig::default()
            },
        ));
    }
    for name in ICON_NAMES {
        icons.push((
            format!("icon:{name}"),
            KeyConfig {
                bg: bg_hex.into(),
                image: Some(PathBuf::from(format!("icon:{name}"))),
                ..KeyConfig::default()
            },
        ));
    }
    // Fill ratio = bbox size / inner rect size, in percent.
    let mut failures = Vec::new();
    for (name, config) in &icons {
        let img = key_image(5, Some(config), &root, &inset);
        let Some((x, y, w, h)) = content_bbox(&img, bg) else {
            failures.push(format!("{name}: no content"));
            continue;
        };
        let gap_x = (x.abs_diff(rect.x), (rect.x + rect.w).abs_diff(x + w));
        let gap_y = (y.abs_diff(rect.y), (rect.y + rect.h).abs_diff(y + h));
        eprintln!(
            "{name:>22} bbox x={x:>2} y={y:>2} w={w:>2} h={h:>2} | gaps L{} R{} T{} B{} | fill w={:>3}% h={:>3}%",
            gap_x.0, gap_x.1, gap_y.0, gap_y.1,
            w * 100 / rect.w,
            h * 100 / rect.h,
        );
        let fits_x = gap_x.0 <= 3 && gap_x.1 <= 3;
        let fits_y = gap_y.0 <= 3 && gap_y.1 <= 3;
        if !(fits_x || fits_y) {
            failures.push(format!(
                "{name}: gaps L{} R{} T{} B{}",
                gap_x.0, gap_x.1, gap_y.0, gap_y.1
            ));
        }
    }
    for word in ["HELLO", "BUILD", "TERMINAL"] {
        let config = KeyConfig {
            label: word.into(),
            bg: bg_hex.into(),
            fg: "#FFFFFF".into(),
            ..KeyConfig::default()
        };
        let img = key_image(5, Some(&config), &root, &inset);
        let (x, y, w, h) = content_bbox(&img, bg).unwrap_or((0, 0, 0, 0));
        eprintln!(
            "{word:>22} bbox x={x:>2} y={y:>2} w={w:>2} h={h:>2} | text width {}% of inner width",
            w * 100 / rect.w
        );
        assert!(w * 100 >= rect.w * 90, "{word}: width {w} of {}", rect.w);
    }
    assert!(
        failures.is_empty(),
        "icons not filling the rect: {failures:#?}"
    );
}
