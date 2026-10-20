//! Profile reload: re-read `profile.toml` and apply it to the running app.
use crate::application::render_screen;
use crate::domain::navigation::NavigationStack;
use crate::domain::profile::Profile;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::profile_store;
use crate::infrastructure::usb::Device;
use crate::presentation::tui::AppState;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// Log each semantic problem in the profile. The layout still loads; bad keys do nothing.
pub fn log_problems(profile: &Profile) {
    for problem in profile.problems() {
        warn!("profile: {problem}");
    }
}

/// Load the edited profile into the running app. A broken or missing file keeps the
/// current layout. The controller never writes the file here. True when a new layout loaded.
pub fn reload_profile(
    app_state: &Arc<Mutex<AppState>>,
    nav: &mut NavigationStack,
    handle: &Device,
    dev_state: &Arc<Mutex<DeviceState>>,
) -> bool {
    match profile_store::read(&profile_store::profile_path()) {
        Ok(Some(profile)) => {
            let count = profile.pages.len();
            let prompt_count = profile.session_prompts().len();
            log_problems(&profile);
            {
                let mut s = app_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                s.total_pages = count;
                s.profile = profile;
                s.auto_page.clamp(count);
            }
            nav.set_main_pages(count);
            nav.set_action_prompts(prompt_count);
            info!("profile reloaded: {count} page(s)");
            render_screen(nav, handle, app_state, dev_state);
            true
        }
        Ok(None) => {
            info!("profile.toml is missing; keeping current layout");
            false
        }
        Err(e) => {
            error!("profile reload failed, keeping current layout: {e}");
            false
        }
    }
}
