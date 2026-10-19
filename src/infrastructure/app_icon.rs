//! Icons of installed macOS apps, looked up by bundle id.
//!
//! This file holds the pure logic: input checks, plist parsing, candidate
//! selection. Process and file calls live in `app_icon/os.rs`.

use std::path::{Path, PathBuf};

#[path = "app_icon/os.rs"]
mod os;

pub use os::app_icon;

/// Smallest icon edge in pixels.
pub const MIN_SIZE: u32 = 16;
/// Largest icon edge in pixels.
pub const MAX_SIZE: u32 = 1024;

/// Validate a request. Returns the size clamped to `MIN_SIZE..=MAX_SIZE`, or `None` for a bad bundle id.
pub fn normalize_request(bundle_id: &str, size: u32) -> Option<u32> {
    is_valid_bundle_id(bundle_id).then(|| size.clamp(MIN_SIZE, MAX_SIZE))
}

/// Bundle ids use only ASCII letters, digits, `.` and `-`. Anything else is
/// rejected before it reaches a command argument.
pub fn is_valid_bundle_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 255
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
}

/// Cache file name for a bundle id at a size, e.g. `com.google.Chrome-85.png`.
pub fn cache_file_name(bundle_id: &str, size: u32) -> String {
    format!("{bundle_id}-{size}.png")
}

/// Pick the app from `mdfind` output. Apps in standard folders win over other hits such as Downloads.
pub fn pick_app(mdfind_stdout: &str, home: Option<&Path>) -> Option<PathBuf> {
    let apps: Vec<PathBuf> = mdfind_stdout
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(".app"))
        .map(PathBuf::from)
        .collect();
    let standard = apps.iter().find(|app| is_standard_app_dir(app, home));
    standard.or(apps.first()).cloned()
}

fn is_standard_app_dir(app: &Path, home: Option<&Path>) -> bool {
    app.starts_with("/Applications")
        || app.starts_with("/System/Applications")
        || home.is_some_and(|h| app.starts_with(h.join("Applications")))
}

/// An icon name must stay inside `Contents/Resources`: no separators, no `..`.
pub fn is_safe_icon_name(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && !name.contains("..")
}

/// Append `.icns` unless the name already ends with it, case-insensitively.
pub fn icns_file_name(name: &str) -> String {
    if name.to_ascii_lowercase().ends_with(".icns") {
        name.to_owned()
    } else {
        format!("{name}.icns")
    }
}

/// Icon file names to try, most specific first. Unsafe names are dropped.
pub fn icon_candidates(info_xml: &str) -> Vec<String> {
    let mut names: Vec<String> = ["CFBundleIconFile", "CFBundleIconName"]
        .into_iter()
        .filter_map(|key| plist_string(info_xml, key))
        .filter(|name| is_safe_icon_name(name))
        .map(|name| icns_file_name(&name))
        .collect();
    names.push("AppIcon.icns".to_owned());
    names
}

/// Read the string value that follows `<key>{key}</key>` in XML plist text.
fn plist_string(xml: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{key}</key>");
    let after_key = xml.split_once(&key_tag)?.1.trim_start();
    let body = after_key.strip_prefix("<string>")?;
    let value = body.get(..body.find("</string>")?)?.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
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
}
