use super::*;

#[test]
fn skips_null_and_empty_bundle_ids() {
    let text = "1) \"Helper\" ASN:0x0-0x1:\n    bundleID=\"[ NULL ]\"\n2) \"Empty\" ASN:0x0-0x2:\n    bundleID=\"\"\n3) \"Finder\" ASN:0x0-0x3:\n    bundleID=\"com.apple.finder\"\n";
    assert_eq!(
        parse_lsappinfo(text),
        vec![("Finder".to_string(), "com.apple.finder".to_string())]
    );
}

#[test]
fn bundle_line_without_a_name_is_ignored() {
    assert!(parse_lsappinfo("    bundleID=\"com.x.y\"\n").is_empty());
    assert!(parse_lsappinfo("x) \"Not an entry\"\n    bundleID=\"com.x.y\"\n").is_empty());
}
