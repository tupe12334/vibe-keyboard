//! The menu bar tray: open the editor, pause the controller, toggle login start, quit.
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_autostart::ManagerExt;

use crate::app::App;

/// Tray items that change text or state while the app runs.
pub struct TrayItems {
    pub menu: Menu<Wry>,
    pub pause: MenuItem<Wry>,
    pub autostart: CheckMenuItem<Wry>,
    /// Shown at the top of the menu only while push-to-talk lacks Accessibility.
    pub accessibility: MenuItem<Wry>,
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
    let accessibility = crate::accessibility::menu_item(app)?;
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
        menu: menu.clone(),
        pause: pause.clone(),
        autostart: autostart.clone(),
        accessibility,
    });

    TrayIconBuilder::new()
        .icon(tauri::include_image!("icons/32x32.png"))
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_editor(app),
            "pause" => toggle_controller(app),
            "autostart" => crate::login_item::toggle_autostart(app),
            crate::accessibility::MENU_ID => crate::accessibility::open_settings(),
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
        // A refusal shows in the editor (device status error). The label stays "Resume".
        let _ = state.start(app);
    }
    set_controller_running(app, state.is_running());
}

/// Set the pause item text to match whether the controller thread is alive.
pub fn set_controller_running(app: &AppHandle, running: bool) {
    let label = if running {
        "Pause controller"
    } else {
        "Resume controller"
    };
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.pause.set_text(label);
    }
}
