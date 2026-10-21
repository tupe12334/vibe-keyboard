use super::*;

#[test]
fn safe_icon_name_rejects_path_escape() {
    assert!(is_safe_icon_name("AppIcon"));
    assert!(!is_safe_icon_name("../../evil"));
    assert!(!is_safe_icon_name("sub/icon"));
    assert!(!is_safe_icon_name("sub\\icon"));
    assert!(!is_safe_icon_name("a..b"));
    assert!(!is_safe_icon_name(""));
}

#[test]
fn icns_file_name_appends_only_when_missing() {
    assert_eq!(icns_file_name("app"), "app.icns");
    assert_eq!(icns_file_name("Logo.ICNS"), "Logo.ICNS");
    assert_eq!(icns_file_name("app.v2"), "app.v2.icns");
}

/// Wrap `<key>`/`<string>` pairs in an XML plist document.
fn xml_plist(body: &str) -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict>\n{body}\n</dict></plist>"
    )
    .into_bytes()
}

#[test]
fn icon_candidates_add_icns_extension_and_fallback() {
    let info = xml_plist(
        "<key>CFBundleIdentifier</key>\n\t<string>com.google.Chrome</string>\n\
        <key>CFBundleIconFile</key>\n\t<string>app</string>\n\
        <key>CFBundleIconName</key>\n\t<string>  </string>",
    );
    assert_eq!(
        icon_candidates(&info),
        vec!["app.icns".to_owned(), "AppIcon.icns".to_owned()]
    );
    let with_ext = xml_plist("<key>CFBundleIconFile</key><string>Logo.icns</string>");
    assert_eq!(
        icon_candidates(&with_ext),
        vec!["Logo.icns".to_owned(), "AppIcon.icns".to_owned()]
    );
}

#[test]
fn icon_candidates_drop_path_escape_values() {
    let evil = xml_plist(
        "<key>CFBundleIconFile</key><string>../../../etc/x</string>\
        <key>CFBundleIconName</key><string>sub\\y</string>",
    );
    assert_eq!(icon_candidates(&evil), vec!["AppIcon.icns".to_owned()]);
}

#[test]
fn icon_candidates_decode_xml_entities() {
    let info = xml_plist("<key>CFBundleIconFile</key><string>R&amp;D</string>");
    assert_eq!(icon_candidates(&info)[0], "R&D.icns");
}

#[test]
fn icon_candidates_ignore_keys_in_nested_dicts() {
    let info = xml_plist(
        "<key>Nested</key><dict><key>CFBundleIconFile</key><string>wrong</string></dict>\
        <key>CFBundleIconName</key><string>right</string>",
    );
    assert_eq!(
        icon_candidates(&info),
        vec!["right.icns".to_owned(), "AppIcon.icns".to_owned()]
    );
}

#[test]
fn icon_candidates_read_binary_plist() {
    let mut info = plist::Dictionary::new();
    info.insert("CFBundleIconFile".to_owned(), "app".into());
    let mut binary = Vec::new();
    plist::Value::Dictionary(info)
        .to_writer_binary(&mut binary)
        .unwrap_or_else(|e| unreachable!("in-memory write: {e}"));
    assert_eq!(
        icon_candidates(&binary),
        vec!["app.icns".to_owned(), "AppIcon.icns".to_owned()]
    );
}

#[test]
fn icon_candidates_fall_back_on_garbage() {
    assert_eq!(
        icon_candidates(b"not a plist"),
        vec!["AppIcon.icns".to_owned()]
    );
}
