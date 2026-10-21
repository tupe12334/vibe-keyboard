#![allow(clippy::expect_used)]

//! The user's real layout, kept as a fixture. A save must never drop any of it. Each key field
//! has its own assertion, so a failure names the field that was lost.

use std::path::{Path, PathBuf};

use vibe_keyboard::domain::profile::{Action, Profile, Widget};
use vibe_keyboard::infrastructure::profile_store;

mod desktop_load_save;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../tests/fixtures/real_profile.toml"
);

fn load_fixture() -> Profile {
    profile_store::read(Path::new(FIXTURE))
        .expect("read fixture")
        .expect("fixture present")
}

/// One assertion per field that a past build dropped or could drop.
fn assert_key_fields(profile: &Profile) {
    assert_eq!(
        profile.display.key_inset.top, 10,
        "[display.key_inset] top was dropped"
    );
    assert_eq!(
        profile.display.side_inset.top, 45,
        "[display.side_inset] top was dropped"
    );
    assert_eq!(
        profile.display.side_inset.bottom, 0,
        "[display.side_inset] bottom was dropped"
    );
    assert_eq!(
        profile.pages[0].keys[&6].hold,
        Some(Action::PushToTalk { key: None }),
        "key 6 hold (push_to_talk) was dropped"
    );
    assert_eq!(
        profile.pages[0].keys[&14].image,
        Some(PathBuf::from("app:com.googlecode.iterm2")),
        "key 14 image (app:com.googlecode.iterm2) was dropped"
    );
    let side = profile.pages[0]
        .side
        .as_ref()
        .expect("side screen was dropped");
    assert_eq!(
        side.widgets,
        vec![Widget::CpuBar, Widget::Temperature, Widget::DiskFree],
        "side widgets were dropped or reordered"
    );
    assert_eq!(profile.prompts.len(), 2, "prompts were dropped");
    assert_eq!(profile.prompts[0].label, "Keep looking", "prompt 1 label");
    assert_eq!(profile.prompts[1].label, "Execute", "prompt 2 label");
}

#[test]
fn the_real_layout_survives_profile_store_save_and_read() {
    let original = load_fixture();
    assert_key_fields(&original);
    let dir = std::env::temp_dir().join(format!("vibe-keyboard-real-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("profile.toml");
    profile_store::save(&path, &original).expect("save");
    let back = profile_store::read(&path).expect("read").expect("present");
    assert_key_fields(&back);
    assert_eq!(back, original, "the layout changed in a save and read");
}
