//! The config guide written next to the profile: `README.md` is kept equal to the bundled text,
//! `.gitignore` is written once.

use std::fs;
use std::path::Path;

use super::write_atomic;

pub(super) const GUIDE_README: &str = include_str!("../../assets/config/README.md");
const GUIDE_GITIGNORE: &str = "state.toml\napp.log\n*.tmp\n.DS_Store\n";

/// Keep `README.md` equal to the bundled text: write it when missing or different.
/// `.gitignore` is written only when missing, so users can extend it.
pub(super) fn write_guide(root: &Path) -> Result<(), String> {
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
