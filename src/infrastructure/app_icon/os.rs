//! Process and file calls for `app_icon`: `mdfind`, `plutil`, `sips`, and the
//! PNG cache. Excluded from the coverage gate in `.cargo-husky/hooks/pre-push`.

use std::path::{Path, PathBuf};
use std::process::Command;

use image::DynamicImage;
use tracing::debug;

use super::icns_render::{render_icns, run_capture};
use super::{cache_file_name, icon_candidates, normalize_request, pick_app};

/// Icon of an installed app by bundle id, scaled to `size`×`size`, e.g. ("com.google.Chrome", 85).
pub fn app_icon(bundle_id: &str, size: u32) -> Option<DynamicImage> {
    let size = normalize_request(bundle_id, size)?;
    let cache = cache_dir()?.join(cache_file_name(bundle_id, size));
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

fn cache_dir() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("vibe-keyboard").join("app-icons"))
}

fn find_app(bundle_id: &str) -> Option<PathBuf> {
    let query = format!("kMDItemCFBundleIdentifier == '{bundle_id}'");
    let stdout = run_capture(Command::new("mdfind").arg(query))?;
    pick_app(
        &String::from_utf8(stdout).ok()?,
        dirs::home_dir().as_deref(),
    )
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

/// Convert a binary or XML plist to XML text with `plutil`.
fn plist_xml(plist: &Path) -> Option<String> {
    let out = run_capture(
        Command::new("plutil")
            .args(["-convert", "xml1", "-o", "-"])
            .arg(plist),
    )?;
    String::from_utf8(out).ok()
}
