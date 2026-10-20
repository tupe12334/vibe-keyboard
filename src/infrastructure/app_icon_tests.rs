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
