use super::*;

#[test]
fn color_parsing_round_trips() {
    assert_eq!(parse_color("#1A2B3C"), Some([0x1A, 0x2B, 0x3C]));
    assert_eq!(parse_color("1A2B3C"), None);
    assert_eq!(parse_color("#+A2B3C"), None);
    assert_eq!(format_color([0x1A, 0x2B, 0x3C]), "#1A2B3C");
}
