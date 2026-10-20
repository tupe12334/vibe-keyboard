//! Icons of installed macOS apps, looked up by bundle id.
//!
//! This file holds the pure logic: input checks, plist parsing, candidate
//! selection. Process and file calls live in `app_icon/os.rs`.

use std::path::{Path, PathBuf};

#[path = "app_icon/icns_render.rs"]
mod icns_render;
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
#[path = "app_icon_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "app_icon_plist_tests.rs"]
mod plist_tests;
