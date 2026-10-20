//! Startup, the other writer of `profile.toml`: a present file is read and never rewritten.
#![allow(clippy::expect_used)]

use crate::editor_preserve_support::FIXTURE;
use vibe_keyboard::infrastructure::profile_store;

#[test]
fn startup_leaves_the_real_layout_file_byte_for_byte() {
    let dir = std::env::temp_dir().join(format!("vibe-keyboard-startup-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("profile.toml");
    std::fs::copy(FIXTURE, &path).expect("place the real layout");
    let before = std::fs::read(&path).expect("read before");
    let loaded = profile_store::ensure_default(&path).expect("startup load");
    assert_eq!(std::fs::read(&path).expect("read after"), before);
    assert_eq!(loaded.display.key_inset.top, 10, "[display.key_inset] top");
    assert_eq!(
        loaded.display.side_inset.top, 45,
        "[display.side_inset] top"
    );
}
