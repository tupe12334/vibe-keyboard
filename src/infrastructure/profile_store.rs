use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tracing::warn;

use crate::domain::profile::Profile;
use crate::infrastructure::persistence::config_dir;

#[path = "profile_images.rs"]
mod profile_images;
pub use profile_images::{import_image, resolve_image};

/// `~/.config/vibe-keyboard/`. Holds the tracked `profile.toml` and `images/`.
pub fn root_dir() -> PathBuf {
    config_dir().join("vibe-keyboard")
}

pub fn profile_path() -> PathBuf {
    root_dir().join("profile.toml")
}

/// Last modification time of the profile at `path`, if it exists.
pub fn modified_at(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

/// Read the profile at `path`. `Ok(None)` when the file is missing. Never writes.
pub fn read(path: &Path) -> Result<Option<Profile>, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    let mut profile: Profile =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    profile.ensure_one_page();
    Ok(Some(profile))
}

/// Startup only: read the profile, or write the legacy layout once when it is missing.
/// Every start also refreshes the config guide next to the profile.
pub fn ensure_default(path: &Path) -> Result<Profile, String> {
    if let Err(e) = path.parent().map_or(Ok(()), write_guide) {
        warn!("could not write config guide: {e}");
    }
    if let Some(profile) = read(path)? {
        Ok(profile)
    } else {
        let profile = Profile::default_from_legacy();
        if let Err(e) = save(path, &profile) {
            warn!("could not write default profile: {e}");
        }
        Ok(profile)
    }
}

const GUIDE_README: &str = include_str!("../../assets/config/README.md");
const GUIDE_GITIGNORE: &str = "state.toml\napp.log\n*.tmp\n.DS_Store\n";

/// Keep `README.md` equal to the bundled text: write it when missing or different.
/// `.gitignore` is written only when missing, so users can extend it.
fn write_guide(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| format!("create {}: {e}", root.display()))?;
    let readme = root.join("README.md");
    if fs::read(&readme).ok().as_deref() != Some(GUIDE_README.as_bytes()) {
        write_atomic(&readme, GUIDE_README.as_bytes())?;
    }
    let ignore = root.join(".gitignore");
    if !ignore.exists() {
        fs::write(&ignore, GUIDE_GITIGNORE)
            .map_err(|e| format!("write {}: {e}", ignore.display()))?;
    }
    Ok(())
}

/// Write the profile to `path`. Output is deterministic and always ends with a newline.
/// Skips the write when the bytes are unchanged.
#[allow(clippy::expect_used, reason = "a Profile always serializes")]
pub fn save(path: &Path, profile: &Profile) -> Result<(), String> {
    let toml = toml::to_string_pretty(profile).expect("profile serializes to TOML");
    let text = format!("{}\n", toml.trim_end());
    if fs::read_to_string(path).is_ok_and(|current| current == text) {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "profile path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    write_atomic(path, text.as_bytes())
}

/// Write `bytes` to `path` through a sibling `<name>.tmp`, then rename it over `path`.
/// A reader never sees a half-written file.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, bytes).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename to {}: {e}", path.display()))
}

#[cfg(test)]
#[path = "profile_store_tests.rs"]
mod tests;
