#![allow(clippy::expect_used)]

use crate::command_harness::{env_lock, mock_app, request, seed};
use serde_json::json;
use tauri::test::get_ipc_response;
use vibe_keyboard::domain::profile::{Profile, SideConfig, Widget};
use vibe_keyboard::infrastructure::profile_store;

/// The side screen the user keeps on a page. It must come back from a save unchanged.
#[test]
fn the_side_screen_survives_a_save_from_the_editor() {
    let _env = env_lock();
    let path = seed("side");
    let (_app, webview) = mock_app();
    let side = SideConfig {
        label: String::new(),
        bg: "#101018".into(),
        fg: "#E6E6E6".into(),
        image: None,
        widgets: vec![Widget::CpuBar, Widget::Temperature, Widget::DiskFree],
        bar_style: None,
    };

    let mut edited: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("load")
        .deserialize()
        .expect("parse layout");
    edited.set_side(0, Some(side.clone()));
    get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": edited })),
    )
    .expect("save accepted");

    let on_disk = profile_store::read(&path).expect("read").expect("present");
    assert_eq!(on_disk.pages[0].side, Some(side.clone()));
    let loaded: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("reload")
        .deserialize()
        .expect("parse layout");
    assert_eq!(loaded.pages[0].side, Some(side));
}
