use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tracing::warn;

use crate::domain::profile::Profile;
use crate::infrastructure::persistence::config_dir;

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

/// Turn a stored image path into a real path. Relative paths resolve against `root`.
pub fn resolve_image(root: &Path, image: &Path) -> PathBuf {
    // `Path::join` keeps an absolute `image` unchanged, so one call covers both cases.
    root.join(image)
}

/// Make `src` usable from the profile. Returns a path relative to `root`.
/// Paths are canonicalized first, so `..` cannot escape `root`. Files outside `root` are
/// copied into `root/images/`, with a numbered name if needed.
pub fn import_image(root: &Path, src: &Path) -> Result<PathBuf, String> {
    let images = root.join("images");
    fs::create_dir_all(&images).map_err(|e| format!("create {}: {e}", images.display()))?;
    // Canonical form lets the `strip_prefix` check see through `..` and symlinks.
    let images = fs::canonicalize(&images).unwrap_or(images);
    let root = images.parent().unwrap_or(&images).to_path_buf();
    let src = fs::canonicalize(src).map_err(|e| format!("read {}: {e}", src.display()))?;
    if let Ok(relative) = src.strip_prefix(&root) {
        return Ok(relative.to_path_buf());
    }
    let name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("no usable file name in {}", src.display()))?;
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    let ext = src.extension().and_then(|e| e.to_str());
    let bytes = fs::read(&src).map_err(|e| format!("read {}: {e}", src.display()))?;
    let mut counter = 0u32;
    loop {
        let candidate = match (counter, ext) {
            (0, _) => name.to_string(),
            (n, Some(ext)) => format!("{stem}-{n}.{ext}"),
            (n, None) => format!("{stem}-{n}"),
        };
        let dest = images.join(&candidate);
        match fs::read(&dest) {
            Ok(existing) if existing == bytes => return Ok(images_relative(candidate)),
            Ok(_) => counter += 1,
            Err(_) => return copy_to(&dest, &bytes, candidate),
        }
    }
}

fn images_relative(candidate: String) -> PathBuf {
    PathBuf::from("images").join(candidate)
}

fn copy_to(dest: &Path, bytes: &[u8], candidate: String) -> Result<PathBuf, String> {
    fs::write(dest, bytes).map_err(|e| format!("write {}: {e}", dest.display()))?;
    Ok(images_relative(candidate))
}

#[cfg(test)]
#[path = "profile_store_tests.rs"]
mod tests;
