//! The menu bar tray: open the editor, pause the controller, toggle login start, quit.
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_autostart::ManagerExt;

use crate::app::App;

/// Tray items that change text or state while the app runs.
pub struct TrayItems {
    pub pause: MenuItem<Wry>,
    pub autostart: CheckMenuItem<Wry>,
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);
    let open = MenuItem::with_id(app, "open", "Open editor", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause controller", true, None::<&str>)?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        "Start at login",
        true,
        autostart_on,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit Vibe Keyboard", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &open,
            &pause,
            &PredefinedMenuItem::separator(app)?,
            &autostart,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    app.manage(TrayItems {
        pause: pause.clone(),
        autostart: autostart.clone(),
    });

    TrayIconBuilder::new()
        .icon(tauri::include_image!("icons/32x32.png"))
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_editor(app),
            "pause" => toggle_controller(app),
            "autostart" => toggle_autostart(app),
            "quit" => {
                app.state::<App>().stop();
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn show_editor(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_controller(app: &AppHandle) {
    let state = app.state::<App>();
    if state.is_running() {
        state.stop();
    } else {
        state.start(app);
    }
    let label = if state.is_running() {
        "Pause controller"
    } else {
        "Resume controller"
    };
    let _ = app.state::<TrayItems>().pause.set_text(label);
}

fn toggle_autostart(app: &AppHandle) {
    let manager = app.autolaunch();
    let enable = !manager.is_enabled().unwrap_or(false);
    let result = if enable {
        manager.enable()
    } else {
        manager.disable()
    };
    if result.is_ok() {
        let _ = app.state::<TrayItems>().autostart.set_checked(enable);
    }
}
