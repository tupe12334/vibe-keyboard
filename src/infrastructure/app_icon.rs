//! Icons of installed macOS apps, looked up by bundle id.
//!
//! Uses only macOS tools (`mdfind`, `plutil`, `sips`) and cached PNGs under
//! `~/Library/Caches/vibe-keyboard/app-icons/`.

use std::path::{Path, PathBuf};
use std::process::Command;

use image::DynamicImage;
use tracing::debug;

/// Icon of an installed app by bundle id, scaled to `size`×`size`, e.g. ("com.google.Chrome", 85).
pub fn app_icon(bundle_id: &str, size: u32) -> Option<DynamicImage> {
    if size == 0 || !is_valid_bundle_id(bundle_id) {
        return None;
    }
    let cache = cache_path(bundle_id, size)?;
    if let Ok(img) = image::open(&cache) {
        return Some(img);
    }
    let app = find_app(bundle_id)?;
    let icns = icns_path(&app)?;
    if !render_icns(&icns, size, &cache) {
        debug!("sips failed for {bundle_id} ({})", icns.display());
        return None;
    }
    image::open(&cache).ok()
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

fn cache_path(bundle_id: &str, size: u32) -> Option<PathBuf> {
    let base = dirs::cache_dir()?;
    Some(
        base.join("vibe-keyboard")
            .join("app-icons")
            .join(format!("{bundle_id}-{size}.png")),
    )
}

fn find_app(bundle_id: &str) -> Option<PathBuf> {
    let query = format!("kMDItemCFBundleIdentifier == '{bundle_id}'");
    let out = Command::new("mdfind").arg(query).output().ok()?;
    let stdout = String::from_utf8(out.stdout).ok()?;
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.ends_with(".app"))
        .map(PathBuf::from)
}

/// Find the `.icns` file that an app bundle names in its `Info.plist`.
fn icns_path(app: &Path) -> Option<PathBuf> {
    let info = plist_xml(&app.join("Contents").join("Info.plist"))?;
    let resources = app.join("Contents").join("Resources");
    icon_candidates(&info)
        .into_iter()
        .map(|name| resources.join(name))
        .find(|path| path.is_file())
}

/// Icon file names to try, most specific first.
fn icon_candidates(info_xml: &str) -> Vec<String> {
    let mut names = Vec::new();
    for key in ["CFBundleIconFile", "CFBundleIconName"] {
        if let Some(name) = plist_string(info_xml, key) {
            let file = if Path::new(&name).extension().is_some() {
                name
            } else {
                format!("{name}.icns")
            };
            names.push(file);
        }
    }
    names.push("AppIcon.icns".to_owned());
    names
}

/// Read the string value that follows `<key>{key}</key>` in XML plist text.
pub fn plist_string(xml: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{key}</key>");
    let after_key = xml.split_once(&key_tag)?.1.trim_start();
    let body = after_key.strip_prefix("<string>")?;
    let value = body.get(..body.find("</string>")?)?.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// Convert a binary or XML plist to XML text with `plutil`.
fn plist_xml(plist: &Path) -> Option<String> {
    let out = Command::new("plutil")
        .args(["-convert", "xml1", "-o", "-"])
        .arg(plist)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

fn render_icns(icns: &Path, size: u32, out: &Path) -> bool {
    let Some(dir) = out.parent() else {
        return false;
    };
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let side = size.to_string();
    Command::new("sips")
        .args(["-s", "format", "png", "--resampleHeightWidth", &side, &side])
        .arg(icns)
        .arg("--out")
        .arg(out)
        .status()
        .is_ok_and(|s| s.success())
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
    fn app_icon_rejects_bad_input_without_running_commands() {
        assert!(app_icon("com.x'; touch /tmp/pwned", 85).is_none());
        assert!(app_icon("com.google.Chrome", 0).is_none());
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
}
