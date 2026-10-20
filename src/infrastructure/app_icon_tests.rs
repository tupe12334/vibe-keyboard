use super::*;

#[test]
fn accepts_plain_bundle_ids() {
    assert!(is_valid_bundle_id("com.google.Chrome"));
    assert!(is_valid_bundle_id("com.googlecode.iterm2"));
    assert!(is_valid_bundle_id("com.apple.finder"));
}

#[test]
fn rejects_shell_metacharacters_and_empty() {
    assert!(!is_valid_bundle_id(""));
    assert!(!is_valid_bundle_id("com.x; rm -rf ~"));
    assert!(!is_valid_bundle_id("com.x'$(id)"));
    assert!(!is_valid_bundle_id("com/x"));
    assert!(!is_valid_bundle_id("com.x\n"));
    assert!(!is_valid_bundle_id(&"a".repeat(256)));
}

#[test]
fn normalize_request_rejects_bad_bundle_id() {
    assert!(normalize_request("com.x'; touch /tmp/pwned", 85).is_none());
}

#[test]
fn normalize_request_clamps_size() {
    assert_eq!(normalize_request("com.google.Chrome", 0), Some(16));
    assert_eq!(normalize_request("com.google.Chrome", 85), Some(85));
    assert_eq!(normalize_request("com.google.Chrome", 4096), Some(1024));
}

#[test]
fn cache_file_name_includes_bundle_id_and_size() {
    assert_eq!(
        cache_file_name("com.google.Chrome", 85),
        "com.google.Chrome-85.png"
    );
}

#[test]
fn pick_app_prefers_standard_folders_over_downloads() {
    let out = "/Users/me/Downloads/Chrome.app\n/Applications/Google Chrome.app\n";
    assert_eq!(
        pick_app(out, Some(Path::new("/Users/me"))),
        Some(PathBuf::from("/Applications/Google Chrome.app"))
    );
}

#[test]
fn pick_app_accepts_home_applications_and_system_applications() {
    let home = Path::new("/Users/me");
    let user = "/Users/me/Downloads/x.app\n/Users/me/Applications/Y.app\n";
    assert_eq!(
        pick_app(user, Some(home)),
        Some(PathBuf::from("/Users/me/Applications/Y.app"))
    );
    let system = "/tmp/x.app\n/System/Applications/Calendar.app\n";
    assert_eq!(
        pick_app(system, None),
        Some(PathBuf::from("/System/Applications/Calendar.app"))
    );
}

#[test]
fn pick_app_falls_back_to_first_hit_or_none() {
    assert_eq!(
        pick_app("/Volumes/Other/A.app\n", None),
        Some(PathBuf::from("/Volumes/Other/A.app"))
    );
    assert_eq!(pick_app("not an app\n", None), None);
    assert_eq!(pick_app("", None), None);
}

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
