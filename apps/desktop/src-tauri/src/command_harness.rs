//! Runs the profile commands through the IPC layer with no window, for the command tests.
#![allow(clippy::expect_used)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY};
use tauri::webview::{InvokeRequest, WebviewWindow, WebviewWindowBuilder};
use tauri::App;
use vibe_keyboard::domain::profile::Profile;
use vibe_keyboard::infrastructure::profile_store;
use vibe_keyboard::presentation::tui::AppState;

use crate::app::App as DesktopApp;

/// The tests set `XDG_CONFIG_HOME`, which is process-wide. Each test holds this while it runs.
pub fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// An IPC request for `cmd` with a JSON body.
pub fn request(cmd: &str, body: serde_json::Value) -> InvokeRequest {
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
pub fn seed(name: &str) -> PathBuf {
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

/// A mock app with the profile commands registered. Keep the app alive while the webview is used.
pub fn mock_app() -> (App<MockRuntime>, WebviewWindow<MockRuntime>) {
    let app = mock_builder()
        .manage(DesktopApp::new(Arc::new(Mutex::new(AppState::new(0)))))
        .invoke_handler(tauri::generate_handler![
            crate::commands::get_profile,
            crate::commands::save_profile
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app");
    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("webview");
    (app, webview)
}
