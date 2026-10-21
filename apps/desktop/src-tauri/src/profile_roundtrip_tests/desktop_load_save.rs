use super::{assert_key_fields, load_fixture};
use crate::command_harness::{env_lock, mock_app, request, seed};
use serde_json::json;
use tauri::test::get_ipc_response;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::profile_store;

#[test]
fn the_real_layout_survives_the_desktop_load_and_save_path() {
    let _env = env_lock();
    let path = seed("real-layout");
    let original = load_fixture();
    profile_store::save(&path, &original).expect("place the real layout");
    let (_app, webview) = mock_app();

    let loaded: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("load")
        .deserialize()
        .expect("parse layout");
    get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": loaded, "force": false })),
    )
    .expect("save accepted");

    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_key_fields(&on_disk);
    assert_eq!(
        on_disk, original,
        "the layout changed in the desktop save path"
    );
}
