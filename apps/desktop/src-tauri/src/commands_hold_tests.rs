#![allow(clippy::expect_used)]

use crate::command_harness::{env_lock, mock_app, request, seed};
use serde_json::json;
use tauri::test::get_ipc_response;
use vibe_keyboard::domain::profile::{Action, KeyConfig};
use vibe_keyboard::infrastructure::profile_store;

/// The editor saves what `get_profile` returned. A hold action on a key must come back unchanged.
#[test]
fn a_hold_action_survives_a_save_from_the_editor() {
    let _env = env_lock();
    let path = seed("hold");
    let mut profile = profile_store::read(&path).expect("read").expect("present");
    let hold = Action::PushToTalk { key: None };
    profile.pages[0].keys.insert(
        6,
        KeyConfig {
            hold: Some(hold.clone()),
            ..KeyConfig::default()
        },
    );
    profile_store::save(&path, &profile).expect("seed hold");
    let (_app, webview) = mock_app();

    let loaded: serde_json::Value = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("load")
        .deserialize()
        .expect("json");
    get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": loaded, "force": false })),
    )
    .expect("save accepted");

    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages[0].keys[&6].hold, Some(hold));
}
