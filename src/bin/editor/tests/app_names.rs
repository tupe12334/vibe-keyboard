use super::*;

#[test]
fn parses_names_and_bundle_ids() {
    let text = "1) \"Finder\" ASN:0x0-0x1:\n    bundleID=\"com.apple.finder\"\n    executable=x\n2) \"Helper\" ASN:0x0-0x2:\n    bundleID=\"[ NULL ]\"\n";
    assert_eq!(
        parse_lsappinfo(text),
        vec![("Finder".to_string(), "com.apple.finder".to_string())]
    );
}
