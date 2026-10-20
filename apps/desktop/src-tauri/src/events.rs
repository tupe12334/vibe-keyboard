//! Controller events forwarded to the webview as Tauri events.
use serde::Serialize;
use specta::Type;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use vibe_keyboard::controller::ControllerEvent;

use crate::app::App;

/// Device came up. Sent by the controller.
#[derive(Clone, Serialize, Type, tauri_specta::Event)]
pub struct DeviceConnected {}

/// Device went away, or the controller stopped.
#[derive(Clone, Serialize, Type, tauri_specta::Event)]
pub struct DeviceDisconnected {}

/// A key went down. `key` is the logical key number, 1–15.
#[derive(Clone, Serialize, Type, tauri_specta::Event)]
pub struct KeyPressed {
    pub key: u8,
}

/// The profile file was read again and is on the deck.
#[derive(Clone, Serialize, Type, tauri_specta::Event)]
pub struct ProfileReloaded {}

/// Keep the device flag current and forward every controller event to the webview.
pub fn on_event(app: &AppHandle, event: ControllerEvent) {
    let state = app.state::<App>();
    // Event emit errors only mean no webview is listening. Nothing to do about them.
    match event {
        ControllerEvent::DeviceReady | ControllerEvent::DeviceRestored => {
            state.device_connected.store(true, Ordering::Relaxed);
            let _ = DeviceConnected {}.emit(app);
        }
        ControllerEvent::DeviceLost | ControllerEvent::Stopped => {
            state.device_connected.store(false, Ordering::Relaxed);
            let _ = DeviceDisconnected {}.emit(app);
        }
        ControllerEvent::KeyPressed(key) => {
            let _ = KeyPressed { key }.emit(app);
        }
        ControllerEvent::ProfileReloaded => {
            let _ = ProfileReloaded {}.emit(app);
        }
    }
}
