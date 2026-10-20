use super::*;
use crate::domain::profile::Profile;

#[test]
fn default_display_is_not_written() {
    let text = toml::to_string_pretty(&Profile::default()).expect("serializes");
    assert!(!text.contains("display"), "{text}");
}

#[test]
fn non_default_display_round_trips() {
    let mut profile = Profile::default();
    profile.display.key_inset.top = 12;
    profile.display.side_inset.bottom = 0;
    let text = toml::to_string_pretty(&profile).expect("serializes");
    let back: Profile = toml::from_str(&text).expect("parses");
    assert_eq!(back, profile);
}

#[test]
fn missing_display_table_means_defaults() {
    let back: Profile = toml::from_str("").expect("parses");
    assert_eq!(back.display, DisplaySettings::default());
    assert_eq!(back.display.key_inset.top, 8);
    assert_eq!(
        back.display.side_inset,
        SideInset {
            top: 30,
            bottom: 40
        }
    );
}

#[test]
fn inner_rect_removes_each_inset() {
    assert_eq!(
        inner_rect(85, 85, 8, 0, 0, 0),
        PixelRect {
            x: 0,
            y: 8,
            w: 85,
            h: 77
        }
    );
    assert_eq!(
        inner_rect(100, 480, 30, 40, 0, 0),
        PixelRect {
            x: 0,
            y: 30,
            w: 100,
            h: 410
        }
    );
    assert_eq!(
        inner_rect(85, 85, 10, 10, 5, 5),
        PixelRect {
            x: 5,
            y: 10,
            w: 75,
            h: 65
        }
    );
}

#[test]
fn oversized_insets_leave_nothing_without_panicking() {
    let rect = inner_rect(85, 85, 90, 90, 90, 90);
    assert_eq!(rect.w, 0);
    assert_eq!(rect.h, 0);
}
