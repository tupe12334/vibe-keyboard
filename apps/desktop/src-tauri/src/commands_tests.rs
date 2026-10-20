#![allow(clippy::expect_used)]

use crate::command_harness::{env_lock, mock_app, request, seed};
use serde_json::json;
use tauri::test::get_ipc_response;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::profile_store;

#[test]
fn the_commands_refuse_an_external_edit_between_load_and_save() {
    let _env = env_lock();
    let path = seed("external");
    let (_app, webview) = mock_app();

    // Load through the command, then save a change: accepted.
    let loaded: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("load")
        .deserialize()
        .expect("parse layout");
    let mut edited = loaded.clone();
    edited.add_page("From the editor");
    get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": edited, "force": false })),
    )
    .expect("first save accepted");

    // Another editor writes the file. Then this window saves its old load: refused.
    let mut external = Profile::default();
    external.add_page("External");
    profile_store::save(&path, &external).expect("external write");
    let stale = get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": loaded, "force": false })),
    );
    let refusal = stale.expect_err("stale save refused");
    assert!(refusal.to_string().contains("conflict"), "{refusal}");
    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages[0].name, "External");
}
