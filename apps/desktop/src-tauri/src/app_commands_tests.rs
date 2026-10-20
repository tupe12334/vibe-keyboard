#![allow(clippy::expect_used)]

use std::path::PathBuf;

use crate::command_harness::{env_lock, mock_app, request, seed};
use serde_json::json;
use tauri::test::get_ipc_response;
use vibe_keyboard::domain::profile::{KeyConfig, Profile};
use vibe_keyboard::infrastructure::profile_store;

const IMAGE: &str = "app:com.googlecode.iterm2";

#[test]
fn an_app_icon_key_survives_a_save_from_the_editor() {
    let _env = env_lock();
    let path = seed("app-icon");
    let (_app, webview) = mock_app();

    // The editor loads the layout, sets key 14 to an app icon, and saves it.
    let mut edited: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("load")
        .deserialize()
        .expect("parse layout");
    edited.pages[0].keys.insert(
        14,
        KeyConfig {
            image: Some(PathBuf::from(IMAGE)),
            ..KeyConfig::default()
        },
    );
    get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": edited, "force": false })),
    )
    .expect("save accepted");

    // The value reads back unchanged, both from the file text and from the parsed layout.
    let text = std::fs::read_to_string(&path).expect("read file");
    assert!(
        text.contains("image = \"app:com.googlecode.iterm2\""),
        "{text}"
    );
    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages[0].keys[&14].image, Some(PathBuf::from(IMAGE)));
}

#[test]
fn check_bundle_id_uses_the_core_rule() {
    assert!(super::check_bundle_id("com.googlecode.iterm2".into()));
    assert!(!super::check_bundle_id("iTerm".into()));
    assert!(!super::check_bundle_id("Google Chrome.app".into()));
}
