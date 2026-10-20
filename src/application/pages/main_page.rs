use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::infrastructure::usb::Device;
use image::{DynamicImage, RgbImage};
use tracing::info;

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::domain::navigation::NavigationStack;
use crate::domain::profile::{SideConfig, SideInset};
use crate::domain::side::{resolve_side, SideContent};
use crate::infrastructure::images::{page_images, side_image};
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::profile_store;
use crate::infrastructure::system_metrics::Metrics;
use crate::infrastructure::usb::{send_button_image, send_side_image};
use crate::presentation::tui;

/// The hard-coded pages from before profiles. Kept only as the reference for `default_from_legacy`.
#[cfg(test)]
pub fn page_actions(page: usize) -> HashMap<u8, ButtonAction> {
    let mut map = HashMap::new();
    match page {
        0 => {
            map.insert(
                1,
                ButtonAction {
                    name: "Centy".into(),
                    title: "List Projects".into(),
                    description: "Show centy projects".into(),
                },
            );
            map.insert(
                2,
                ButtonAction {
                    name: "Terminal".into(),
                    title: "Open Terminal".into(),
                    description: "Launch a new terminal window".into(),
                },
            );
            map.insert(
                3,
                ButtonAction {
                    name: "Claude".into(),
                    title: "Open Claude".into(),
                    description: "Launch terminal with claude --allow-dangerously-skip-permissions"
                        .into(),
                },
            );
            map.insert(
                4,
                ButtonAction {
                    name: "Centy Web".into(),
                    title: "Open Centy".into(),
                    description: "Open app.centy.io in Chrome".into(),
                },
            );
        }
        1 => {
            map.insert(
                1,
                ButtonAction {
                    name: "NumPad".into(),
                    title: "Input Number".into(),
                    description: "Open numpad entry screen".into(),
                },
            );
            map.insert(
                14,
                ButtonAction {
                    name: "Log File".into(),
                    title: "Open Log".into(),
                    description: "Open app.log in VS Code".into(),
                },
            );
            map.insert(
                15,
                ButtonAction {
                    name: "VSCode Config".into(),
                    title: "Open Config".into(),
                    description: "Open config file in VS Code".into(),
                },
            );
        }
        _ => {}
    }
    map
}

pub fn render_main_page(
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let (keys, key_inset) = {
        let s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let keys = s
            .profile
            .pages
            .get(page)
            .map(|p| p.keys.clone())
            .unwrap_or_default();
        (keys, s.profile.display.key_inset)
    };
    let actions: HashMap<u8, ButtonAction> = keys
        .iter()
        .map(|(key, cfg)| {
            (
                *key,
                ButtonAction {
                    name: cfg.label.clone(),
                    title: cfg.action.kind().label().into(),
                    description: String::new(),
                },
            )
        })
        .collect();
    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::MainPage { page };
    }
    {
        let mut ds = dev_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        ds.current_page = page;
        ds.save();
    }
    // The same images the editor previews. Keys 11–13 are the navigation art.
    let root = profile_store::root_dir();
    for (key, img) in page_images(&keys, &root, &key_inset) {
        send_button_image(handle, key, DynamicImage::ImageRgb8(img));
    }
    info!("page {}: {} key(s)", page + 1, keys.len());
}

/// The side screen `nav` shows, with the metrics it reads. None keeps the side screen.
/// The home page is the main page the stack opened from.
fn side_to_show(
    state: &Arc<Mutex<tui::AppState>>,
    nav: &NavigationStack,
) -> Option<(SideConfig, Metrics, SideInset)> {
    let s = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let page_side = s
        .profile
        .pages
        .get(nav.home_page())
        .and_then(|p| p.side.clone());
    match resolve_side(page_side, nav.current().side()) {
        SideContent::Show(side) => Some((side, s.metrics.clone(), s.profile.display.side_inset)),
        SideContent::Keep => None,
    }
}

/// Sends `frame` unless the device already shows it. `side_frame` is cleared on every full
/// redraw, because that redraw blanks the side screen too.
fn send_side_frame(handle: &Device, state: &Arc<Mutex<tui::AppState>>, frame: &RgbImage) {
    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if s.side_frame.as_deref() == Some(frame.as_raw().as_slice()) {
            return;
        }
        s.side_frame = Some(frame.as_raw().clone());
    }
    send_side_image(handle, DynamicImage::ImageRgb8(frame.clone()));
}

/// Draws the side screen `nav` shows. Called after every full redraw.
pub fn render_side(handle: &Device, state: &Arc<Mutex<tui::AppState>>, nav: &NavigationStack) {
    let Some((side, metrics, inset)) = side_to_show(state, nav) else {
        return;
    };
    let frame = side_image(Some(&side), &profile_store::root_dir(), &metrics, &inset);
    send_side_frame(handle, state, &frame);
}

/// Re-sends the side screen when it has widgets and their frame changed. Called on the
/// controller tick, on main pages and on built-in screens.
pub fn refresh_side_widgets(
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    nav: &NavigationStack,
) {
    let Some((side, metrics, inset)) =
        side_to_show(state, nav).filter(|(side, _, _)| !side.widgets.is_empty())
    else {
        return;
    };
    let frame = side_image(Some(&side), &profile_store::root_dir(), &metrics, &inset);
    send_side_frame(handle, state, &frame);
}

#[cfg(test)]
#[path = "main_page_tests.rs"]
mod tests;
