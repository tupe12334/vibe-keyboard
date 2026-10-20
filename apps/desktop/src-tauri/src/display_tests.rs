#![allow(clippy::expect_used)]

use crate::command_harness::{env_lock, mock_app, request, seed};
use serde_json::json;
use tauri::test::get_ipc_response;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::profile_store;

/// The user's non-default insets. A save must never drop them.
fn with_insets() -> Profile {
    let mut profile = Profile::default();
    profile.add_page("Start");
    profile.display.key_inset.top = 10;
    profile.display.side_inset.top = 45;
    profile.display.side_inset.bottom = 0;
    profile
}

fn assert_insets(profile: &Profile) {
    assert_eq!(profile.display.key_inset.top, 10);
    assert_eq!(profile.display.side_inset.top, 45);
    assert_eq!(profile.display.side_inset.bottom, 0);
}

#[test]
fn insets_survive_profile_store_save_and_read() {
    let dir =
        std::env::temp_dir().join(format!("vibe-keyboard-insets-store-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create dir");
    let path = dir.join("profile.toml");
    let profile = with_insets();
    profile_store::save(&path, &profile).expect("save");
    let back = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(back.display, profile.display);
    assert_insets(&back);
}

#[test]
fn insets_survive_the_desktop_load_and_save_path() {
    let _env = env_lock();
    let path = seed("insets");
    let (_app, webview) = mock_app();

    let mut edited: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("load")
        .deserialize()
        .expect("parse layout");
    edited.display = with_insets().display;
    get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": edited, "force": false })),
    )
    .expect("save accepted");

    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.display, with_insets().display);
    assert_insets(&on_disk);
    let reloaded: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("reload")
        .deserialize()
        .expect("parse layout");
    assert_insets(&reloaded);
}
