use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::infrastructure::usb::Device;
use image::DynamicImage;
use tracing::info;

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::infrastructure::images::page_images;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::profile_store;
use crate::infrastructure::usb::send_button_image;
use crate::presentation::tui;

mod side_frame;
pub use side_frame::{refresh_side_widgets, render_side};

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
        send_button_image(handle, key, &DynamicImage::ImageRgb8(img));
    }
    info!("page {}: {} key(s)", page + 1, keys.len());
}

#[cfg(test)]
#[path = "main_page_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "main_page_profile_tests.rs"]
mod profile_tests;
