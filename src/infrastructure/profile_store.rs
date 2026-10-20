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
pub fn save(path: &Path, profile: &Profile) -> Result<(), String> {
    // A `Profile` made of strings, maps and integers always serializes.
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
mod tests {
    use super::*;
    use crate::domain::profile::{Action, KeyConfig};
    use std::os::unix::fs::PermissionsExt;

    /// Root ignores file modes, so permission-denied cases cannot fail as expected.
    fn running_as_root() -> bool {
        std::env::var("USER").is_ok_and(|user| user == "root")
    }

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_or(0, |d| d.subsec_nanos());
            let dir =
                std::env::temp_dir().join(format!("vibe-kb-{tag}-{}-{nanos}", std::process::id()));
            let _ = fs::create_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn read_ok(path: &Path) -> Profile {
        read(path).ok().flatten().unwrap_or_default()
    }

    #[test]
    fn paths_point_into_the_vibe_keyboard_config_dir() {
        assert!(profile_path().ends_with("vibe-keyboard/profile.toml"));
        assert!(root_dir().ends_with("vibe-keyboard"));
    }

    #[test]
    fn startup_keeps_an_existing_profile() {
        let dir = TempDir::new("exists");
        let path = dir.0.join("profile.toml");
        let mut edited = Profile::default_from_legacy();
        edited.add_page("Extra");
        save(&path, &edited).unwrap_or_default();
        assert_eq!(ensure_default(&path).ok(), Some(edited));
    }

    #[test]
    fn startup_writes_legacy_layout_when_missing() {
        let dir = TempDir::new("first-run");
        let path = dir.0.join("profile.toml");
        assert_eq!(
            ensure_default(&path).ok(),
            Some(Profile::default_from_legacy())
        );
        assert!(path.exists());
    }

    #[test]
    fn reload_read_never_creates_the_file() {
        let dir = TempDir::new("no-create");
        let path = dir.0.join("profile.toml");
        assert_eq!(read(&path), Ok(None));
        assert!(!path.exists());
    }

    #[test]
    fn load_save_is_byte_identical_and_stable() {
        let dir = TempDir::new("bytes");
        let path = dir.0.join("profile.toml");
        save(&path, &Profile::default_from_legacy()).unwrap_or_default();
        let first = fs::read_to_string(&path).unwrap_or_default();

        let loaded = read_ok(&path);
        save(&path, &loaded).unwrap_or_default();
        let second = fs::read_to_string(&path).unwrap_or_default();
        save(&path, &loaded).unwrap_or_default();
        let third = fs::read_to_string(&path).unwrap_or_default();

        assert_eq!(first, second);
        assert_eq!(second, third);
        assert!(first.ends_with('\n'));
    }

    #[test]
    fn malformed_file_reports_line_and_column() {
        let dir = TempDir::new("bad");
        let path = dir.0.join("profile.toml");
        fs::write(&path, "[[pages]\nname = \"x\"\n").unwrap_or_default();
        let err = read(&path).err().unwrap_or_default();
        assert!(err.contains("line"), "error was: {err}");
    }

    #[test]
    fn relative_image_resolves_against_root() {
        let root = Path::new("/cfg/vibe-keyboard");
        assert_eq!(
            resolve_image(root, Path::new("images/a.png")),
            PathBuf::from("/cfg/vibe-keyboard/images/a.png")
        );
        assert_eq!(
            resolve_image(root, Path::new("/abs/b.png")),
            PathBuf::from("/abs/b.png")
        );
    }

    #[test]
    fn import_copies_outside_files_and_keeps_inside_paths() {
        let root = TempDir::new("root");
        let outside = TempDir::new("outside");
        let src = outside.0.join("pic.png");
        fs::write(&src, b"one").unwrap_or_default();

        let rel = import_image(&root.0, &src).unwrap_or_default();
        assert_eq!(rel, PathBuf::from("images/pic.png"));
        assert_eq!(fs::read(root.0.join(&rel)).unwrap_or_default(), b"one");

        // Same bytes reuse the copy; different bytes get a numbered name.
        assert_eq!(import_image(&root.0, &src).unwrap_or_default(), rel);
        fs::write(&src, b"two").unwrap_or_default();
        let second = import_image(&root.0, &src).unwrap_or_default();
        assert_eq!(second, PathBuf::from("images/pic-1.png"));

        let inside = root.0.join("images/pic.png");
        assert_eq!(import_image(&root.0, &inside).unwrap_or_default(), rel);
    }

    #[test]
    fn import_resolves_parent_components_before_checking_the_root() {
        let root = TempDir::new("dotdot");
        let images = root.0.join("images");
        fs::create_dir_all(&images).unwrap_or_default();
        fs::write(images.join("a.png"), b"a").unwrap_or_default();
        let sneaky = root
            .0
            .join("images")
            .join("..")
            .join("images")
            .join("a.png");
        assert_eq!(
            import_image(&root.0, &sneaky).unwrap_or_default(),
            PathBuf::from("images/a.png")
        );
    }

    #[test]
    fn saved_profile_keeps_relative_image_paths() {
        let dir = TempDir::new("rel");
        let path = dir.0.join("profile.toml");
        let mut profile = Profile::default_from_legacy();
        profile.set_key(
            0,
            5,
            KeyConfig {
                image: Some(PathBuf::from("images/claude.png")),
                action: Action::None,
                ..KeyConfig::default()
            },
        );
        save(&path, &profile).unwrap_or_default();
        let text = fs::read_to_string(&path).unwrap_or_default();
        assert!(text.contains("image = \"images/claude.png\""));
        assert!(!text.contains("/Users/"));
    }

    #[test]
    fn io_failures_are_reported_not_panicked() {
        let dir = TempDir::new("io");
        let profile = Profile::default_from_legacy();
        // A directory where the file should be: read fails, rename fails, temp write fails.
        let as_dir = dir.0.join("as-dir.toml");
        fs::create_dir_all(&as_dir).unwrap_or_default();
        assert!(read(&as_dir).is_err());
        assert!(save(&as_dir, &profile).is_err());
        let blocked = dir.0.join("blocked.toml");
        fs::create_dir_all(blocked.with_extension("toml.tmp")).unwrap_or_default();
        assert!(save(&blocked, &profile).is_err());
        // A file where the parent directory should be.
        assert!(ensure_default(&as_dir).is_err());
        let file_parent = dir.0.join("file");
        fs::write(&file_parent, b"x").unwrap_or_default();
        assert!(save(&file_parent.join("p.toml"), &profile).is_err());
        assert!(save(Path::new("/"), &profile).is_err());
        // Missing file, but the default cannot be written: still a usable profile.
        let q = dir.0.join("q.toml");
        fs::create_dir_all(q.with_extension("toml.tmp")).unwrap_or_default();
        assert!(ensure_default(&q).is_ok());
    }

    #[test]
    fn loading_never_rewrites_an_existing_profile() {
        // The controller only reads the profile. Runtime state lives in state.toml.
        let dir = TempDir::new("readonly");
        let path = dir.0.join("profile.toml");
        save(&path, &Profile::default_from_legacy()).unwrap_or_default();
        let before = fs::read(&path).unwrap_or_default();
        let mtime_before = modified_at(&path);
        for _ in 0..3 {
            assert_eq!(read_ok(&path), Profile::default_from_legacy());
        }
        assert_eq!(fs::read(&path).unwrap_or_default(), before);
        assert_eq!(modified_at(&path), mtime_before);
        assert_eq!(modified_at(&dir.0.join("gone.toml")), None);
    }

    #[test]
    fn missing_guide_is_written_on_first_run() {
        let dir = TempDir::new("guide");
        let _ = ensure_default(&dir.0.join("profile.toml"));
        let readme = fs::read_to_string(dir.0.join("README.md")).unwrap_or_default();
        assert_eq!(readme, GUIDE_README);
        assert!(readme.starts_with("<!-- Generated by vibe-keyboard"));
        assert!(readme.contains("https://github.com/tupe12334/vibe-keyboard"));
        let ignore = fs::read_to_string(dir.0.join(".gitignore")).unwrap_or_default();
        assert!(ignore.lines().any(|line| line == "state.toml"));
        assert!(ignore.lines().any(|line| line == "app.log"));
    }

    #[test]
    fn different_readme_is_overwritten_on_start() {
        let dir = TempDir::new("guide-diff");
        fs::write(dir.0.join("README.md"), "old text").unwrap_or_default();
        let _ = ensure_default(&dir.0.join("profile.toml"));
        assert_eq!(
            fs::read_to_string(dir.0.join("README.md")).unwrap_or_default(),
            GUIDE_README
        );
    }

    #[test]
    fn identical_readme_is_untouched() {
        let dir = TempDir::new("guide-same");
        let path = dir.0.join("profile.toml");
        let _ = ensure_default(&path);
        let readme = dir.0.join("README.md");
        let before = modified_at(&readme);
        std::thread::sleep(std::time::Duration::from_millis(20));
        let _ = ensure_default(&path);
        assert_eq!(modified_at(&readme), before);
    }

    #[test]
    fn existing_gitignore_is_never_overwritten() {
        let dir = TempDir::new("guide-keep");
        fs::write(dir.0.join(".gitignore"), "mine").unwrap_or_default();
        let _ = ensure_default(&dir.0.join("profile.toml"));
        assert_eq!(
            fs::read_to_string(dir.0.join(".gitignore")).unwrap_or_default(),
            "mine"
        );
    }

    #[test]
    fn guide_failures_are_reported_not_panicked() {
        let dir = TempDir::new("guide-err");
        // Root is a file: the directory cannot be created.
        let file_root = dir.0.join("file");
        fs::write(&file_root, b"x").unwrap_or_default();
        assert!(write_guide(&file_root).is_err());
        // Read-only cases: root ignores file modes, so they only apply to non-root users.
        let root = running_as_root();
        // Read-only root: the profile and the guide both fail, startup still succeeds.
        let locked = dir.0.join("locked");
        fs::create_dir_all(&locked).unwrap_or_default();
        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o500));
        let refused = write_guide(&locked).is_err();
        let startup = ensure_default(&locked.join("profile.toml")).is_ok();
        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o755));
        assert!(refused || root);
        assert!(startup);
        // README already current, read-only root: the .gitignore write fails.
        let current = dir.0.join("current");
        fs::create_dir_all(&current).unwrap_or_default();
        fs::write(current.join("README.md"), GUIDE_README).unwrap_or_default();
        let _ = fs::set_permissions(&current, fs::Permissions::from_mode(0o500));
        let no_ignore = write_guide(&current).is_err();
        let _ = fs::set_permissions(&current, fs::Permissions::from_mode(0o755));
        assert!(no_ignore || root);
    }

    #[test]
    fn import_handles_missing_names_and_extensionless_files() {
        let root = TempDir::new("imp-root");
        let outside = TempDir::new("imp-out");
        // A file that cannot be read is an error, not an empty copy.
        let locked = outside.0.join("locked.png");
        fs::write(&locked, b"secret").unwrap_or_default();
        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o000));
        let refused = import_image(&root.0, &locked).is_err();
        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o644));
        assert!(refused || running_as_root());
        assert!(import_image(&root.0, Path::new("/")).is_err());
        assert!(import_image(&root.0, &outside.0.join("missing.png")).is_err());

        let plain = outside.0.join("icon");
        fs::write(&plain, b"a").unwrap_or_default();
        let first = import_image(&root.0, &plain).unwrap_or_default();
        assert_eq!(first, PathBuf::from("images/icon"));
        fs::write(&plain, b"b").unwrap_or_default();
        assert_eq!(
            import_image(&root.0, &plain).unwrap_or_default(),
            PathBuf::from("images/icon-1")
        );

        // images/ exists as a file, so creating it fails.
        let blocked = TempDir::new("imp-blocked");
        fs::write(blocked.0.join("images"), b"x").unwrap_or_default();
        assert!(import_image(&blocked.0, &plain).is_err());

        // A directory named like the target makes the write fail.
        let clash = TempDir::new("imp-clash");
        fs::create_dir_all(clash.0.join("images/icon")).unwrap_or_default();
        assert!(import_image(&clash.0, &plain).is_err());
    }
}
