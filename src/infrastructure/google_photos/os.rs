//! HTTP calls to Google Photos. Excluded from the coverage gate. The link rule and the page
//! parsing live in `google_photos.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{cover_photo_url, is_share_link};
use crate::infrastructure::profile_store::import_image;

/// Longest wait for one request. The share page is about 1 MB.
const TIMEOUT: Duration = Duration::from_secs(15);

/// Download the cover photo of a public share link into `root/images/`. Returns the path to
/// store in the key's `image`, relative to `root`.
pub fn import_shared_photo(root: &Path, link: &str) -> Result<PathBuf, String> {
    if !is_share_link(link) {
        return Err(
            "Not a Google Photos share link (photos.app.goo.gl/… or photos.google.com/share/…)"
                .into(),
        );
    }
    let agent = ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .build(),
    );
    let html = agent
        .get(link.trim())
        // Without a browser user agent the page omits the og:image tag.
        .header("User-Agent", "Mozilla/5.0")
        .call()
        .and_then(|mut r| r.body_mut().read_to_string())
        .map_err(|e| format!("open link: {e}"))?;
    let url =
        cover_photo_url(&html).ok_or("No photo on that page. Is the album shared by link?")?;
    let bytes = agent
        .get(&url)
        .call()
        .and_then(|mut r| r.body_mut().read_to_vec())
        .map_err(|e| format!("download photo: {e}"))?;
    image::guess_format(&bytes).map_err(|e| format!("not an image: {e}"))?;
    // Go through `import_image` so the name and the dedup match a file picked from disk.
    let dir = std::env::temp_dir().join(format!("vibe-keyboard-photo-{}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let file = dir.join("google-photo.jpg");
    fs::write(&file, &bytes).map_err(|e| format!("write {}: {e}", file.display()))?;
    let imported = import_image(root, &file);
    let _ = fs::remove_dir_all(&dir);
    imported
}
