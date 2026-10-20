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

const INFO_XML: &str =
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict>\n\
    <key>CFBundleIdentifier</key>\n\t<string>com.google.Chrome</string>\n\
    <key>CFBundleIconFile</key>\n\t<string>app</string>\n\
    <key>CFBundleIconName</key>\n\t<string>  </string>\n\
    </dict></plist>";

#[test]
fn plist_string_reads_value_after_key() {
    assert_eq!(
        plist_string(INFO_XML, "CFBundleIdentifier").as_deref(),
        Some("com.google.Chrome")
    );
    assert_eq!(
        plist_string(INFO_XML, "CFBundleIconFile").as_deref(),
        Some("app")
    );
}

#[test]
fn plist_string_ignores_empty_and_missing_keys() {
    assert_eq!(plist_string(INFO_XML, "CFBundleIconName"), None);
    assert_eq!(plist_string(INFO_XML, "NoSuchKey"), None);
}

#[test]
fn icon_candidates_add_icns_extension_and_fallback() {
    assert_eq!(
        icon_candidates(INFO_XML),
        vec!["app.icns".to_owned(), "AppIcon.icns".to_owned()]
    );
    let with_ext = "<key>CFBundleIconFile</key><string>Logo.icns</string>";
    assert_eq!(
        icon_candidates(with_ext),
        vec!["Logo.icns".to_owned(), "AppIcon.icns".to_owned()]
    );
}

#[test]
fn icon_candidates_drop_path_escape_values() {
    let evil = "<key>CFBundleIconFile</key><string>../../../etc/x</string>\
        <key>CFBundleIconName</key><string>sub\\y</string>";
    assert_eq!(icon_candidates(evil), vec!["AppIcon.icns".to_owned()]);
}
