//! Shows the editor at launch when `VK_OPEN_EDITOR=1`. Same path as the tray's "Open editor".
use tauri::AppHandle;

use crate::tray::show_editor;

const ENV_VAR: &str = "VK_OPEN_EDITOR";

/// True only for the exact value `1`. Any other value, or none, keeps the editor hidden.
pub fn open_editor_at_launch(value: Option<&str>) -> bool {
    value == Some("1")
}

/// Called on `RunEvent::Ready`, after setup and the event loop start. Calling it from setup showed nothing.
pub fn open_at_launch(app: &AppHandle) {
    if open_editor_at_launch(std::env::var(ENV_VAR).ok().as_deref()) {
        tracing::info!("editor: opened at launch (VK_OPEN_EDITOR=1)");
        show_editor(app);
    }
}

#[cfg(test)]
mod tests {
    use super::open_editor_at_launch;

    #[test]
    fn only_the_value_one_opens_the_editor_at_launch() {
        assert!(open_editor_at_launch(Some("1")));
        assert!(!open_editor_at_launch(None));
        assert!(!open_editor_at_launch(Some("")));
        assert!(!open_editor_at_launch(Some("0")));
        assert!(!open_editor_at_launch(Some("true")));
    }
}
