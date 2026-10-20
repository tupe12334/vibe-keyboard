#![allow(clippy::expect_used)]

use std::path::PathBuf;

use crate::command_harness::{env_lock, request, seed};
use serde_json::json;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets};
use tauri::webview::WebviewWindowBuilder;
use vibe_keyboard::domain::profile::{KeyConfig, Profile};

fn call(cmd: &str, body: serde_json::Value) -> Result<serde_json::Value, serde_json::Value> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            super::import_google_photo,
            crate::key_commands::key_preview
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app");
    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview");
    get_ipc_response(&webview, request(cmd, body)).map(|r| r.deserialize().expect("json"))
}

#[test]
fn a_link_that_is_not_google_photos_is_refused_without_a_request() {
    let _env = env_lock();
    seed("photo-refused");
    let reply = call(
        "import_google_photo",
        json!({ "link": "https://example.com/a.jpg" }),
    );
    let error = reply.expect_err("refused").to_string();
    assert!(error.contains("Not a Google Photos share link"), "{error}");
}

/// Real request to a public sample album, then the key preview the editor shows.
/// Run with `cargo test -- --ignored photo`.
#[test]
#[ignore = "network"]
fn a_shared_album_photo_becomes_the_key_image() {
    let _env = env_lock();
    seed("photo-album");
    let link = "https://photos.app.goo.gl/JEadX4KBqViVFXdm9";
    let image = call("import_google_photo", json!({ "link": link })).expect("import");
    let image = image.as_str().expect("path").to_string();
    assert!(image.starts_with("images/google-photo"), "{image}");

    let mut profile = Profile::default();
    profile.add_page("Start");
    let key = KeyConfig {
        image: Some(PathBuf::from(&image)),
        ..KeyConfig::default()
    };
    profile.pages[0].keys.insert(1, key);
    let preview = call(
        "key_preview",
        json!({ "profile": profile, "page": 0, "key": 1 }),
    )
    .expect("preview");
    let url = preview.as_str().expect("data url");
    assert!(url.starts_with("data:image/png;base64,"));
    if let Some(out) = std::env::var_os("PHOTO_PREVIEW_OUT") {
        std::fs::write(out, url).expect("write preview");
    }
}
