use super::*;

use crate::domain::profile::BarStyle;

#[test]
fn invalid_style_fields_fall_back_to_the_default() {
    let style = BarStyle {
        fill: Some("red".into()),
        track: Some("#12".into()),
        thickness: Some(999),
        label_color: Some("nope".into()),
        ..BarStyle::default()
    };
    let look = BarLook::resolve(Some(&style), [9, 9, 9]);
    assert_eq!(look.fill, None);
    assert_eq!(look.track, TRACK);
    assert_eq!(look.width, crate::domain::profile::BAR_THICKNESS_MAX);
    assert_eq!(look.value_color, Some([9, 9, 9]));
}
