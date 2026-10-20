//! Image paths for the profile: resolve stored paths, and copy imported files into `images/`.

use std::fs;
use std::path::{Path, PathBuf};

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
