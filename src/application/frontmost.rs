//! Shows the main page that lists the app in front, when the user has not picked one.

use std::sync::{Arc, Mutex};

use tracing::info;

use super::{lock, render_screen};
use crate::domain::navigation::NavigationStack;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::usb::Device;
use crate::presentation::tui;

/// The app in front changed to `bundle_id`. Show the page that lists it, or return
/// to the manual page when no page does. Does nothing while a sub-screen is open.
pub fn handle_frontmost(
    bundle_id: &str,
    nav: &mut NavigationStack,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let Some(current) = nav.main_page() else {
        return;
    };
    let decision = {
        let mut s = lock(state);
        let target = s.profile.page_for_app(bundle_id);
        s.auto_page.on_frontmost(true, current, target)
    };
    if let Some(page) = decision {
        info!("app {bundle_id} in front: showing page {}", page + 1);
        nav.go_to_page(page);
        render_screen(nav, handle, state, dev_state);
    }
}
