#![allow(clippy::expect_used)]

use crate::app::App;
use crate::commands::{get_profile, save_profile};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::{InvokeRequest, WebviewWindowBuilder};
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::presentation::tui::AppState;

fn request(cmd: &str, body: serde_json::Value) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: "tauri://localhost".parse().expect("url"),
        body: InvokeBody::Json(body),
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_string(),
    }
}

/// Points the profile at a fresh temp dir, with one saved page. Returns the profile file path.
fn seed(name: &str) -> std::path::PathBuf {
    let config =
        std::env::temp_dir().join(format!("vibe-keyboard-cmd-{name}-{}", std::process::id()));
    std::env::set_var("XDG_CONFIG_HOME", &config);
    let path = profile_store::profile_path();
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create dir");
    let mut profile = Profile::default();
    profile.add_page("Start");
    profile_store::save(&path, &profile).expect("seed");
    path
}

#[test]
fn the_commands_refuse_an_external_edit_between_load_and_save() {
    let path = seed("external");
    let app = mock_builder()
        .manage(App::new(Arc::new(Mutex::new(AppState::new(0)))))
        .invoke_handler(tauri::generate_handler![get_profile, save_profile])
        .build(mock_context(noop_assets()))
        .expect("mock app");
    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview");

    // Load through the command, then save a change: accepted.
    let loaded: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("load")
        .deserialize()
        .expect("parse layout");
    let mut edited = loaded.clone();
    edited.add_page("From the editor");
    get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": edited })),
    )
    .expect("first save accepted");

    // Another editor writes the file. Then this window saves its old load: refused.
    let mut external = Profile::default();
    external.add_page("External");
    profile_store::save(&path, &external).expect("external write");
    let stale = get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": loaded })),
    );
    let refusal = stale.expect_err("stale save refused");
    assert!(refusal.to_string().contains("changed on disk"), "{refusal}");
    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages[0].name, "External");
}
