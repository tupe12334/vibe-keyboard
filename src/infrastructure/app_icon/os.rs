//! Process and file calls for `app_icon`: `mdfind`, `plutil`, `sips`, and the
//! PNG cache. Excluded from the coverage gate in `.cargo-husky/hooks/pre-push`.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use image::DynamicImage;
use tracing::debug;

use super::{cache_file_name, icon_candidates, normalize_request, pick_app};

/// Limit for each macOS tool call.
const TOOL_TIMEOUT: Duration = Duration::from_secs(3);

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

/// Render `icns` to a square PNG at `out`. Writes a temp file first and renames it,
/// so a killed run leaves no partial PNG at the cache path.
fn render_icns(icns: &Path, size: u32, out: &Path) -> bool {
    let Some(dir) = out.parent() else {
        return false;
    };
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let tmp = out.with_extension(format!("png.{}.part", std::process::id()));
    let side = size.to_string();
    let rendered = run_capture(
        Command::new("sips")
            .args([
                "-s",
                "format",
                "png",
                "--resampleHeightWidth",
                side.as_str(),
                side.as_str(),
            ])
            .arg(icns)
            .arg("--out")
            .arg(&tmp),
    )
    .is_some();
    if rendered && std::fs::rename(&tmp, out).is_ok() {
        return true;
    }
    let _ = std::fs::remove_file(&tmp);
    false
}

/// Run a tool, collect its stdout, and kill it after `TOOL_TIMEOUT`.
/// Returns `None` on spawn failure, non-zero exit, or timeout.
fn run_capture(cmd: &mut Command) -> Option<Vec<u8>> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    // Read on a thread so a large output cannot block the child on a full pipe.
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).map(|_| buf)
    });
    let deadline = Instant::now() + TOOL_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().ok()? {
            let buf = reader.join().ok()?.ok()?;
            return status.success().then_some(buf);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            debug!("tool timed out, killed");
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
