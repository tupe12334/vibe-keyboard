//! The action page for one Moadim routine: key 1 trigger, key 2 enable or disable, key 3 snooze,
//! key 4 last run. Key 11 (out) is drawn by the page renderer.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::domain::moadim::{Flash, Routine};
use crate::infrastructure::images::label_image;
use crate::infrastructure::moadim::{last_run_label, now_secs};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// How long a pressed key shows its flash label.
pub const MOADIM_FLASH: Duration = Duration::from_millis(1500);

pub fn render_moadim_routine(
    routine: &Routine,
    flash: Option<&Flash>,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let inset = super::super::key_tiles::key_inset(state);
    let flashed =
        |key: u8, normal: &'static str| flash.filter(|f| f.key == key).map_or(normal, |f| f.label);
    let (toggle_name, toggle_word) = if routine.enabled {
        ("Disable", "DISABLE")
    } else {
        ("Enable", "ENABLE")
    };
    let buttons = [
        (1, "Trigger", "Run this routine now", flashed(1, "TRIGGER")),
        (
            2,
            toggle_name,
            "Turn this routine on or off",
            flashed(2, toggle_word),
        ),
        (
            3,
            "Snooze",
            "Skip scheduled runs for one hour",
            flashed(3, "SNOOZE 1H"),
        ),
    ];

    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = Vec::new();
    for (key, name, description, word) in buttons {
        actions.insert(
            key,
            ButtonAction {
                name: name.into(),
                title: name.into(),
                description: description.into(),
            },
        );
        tiles.push((key, label_image(word, &inset)));
    }
    tiles.push((
        4,
        label_image(
            &format!("LAST {}", last_run_label(routine, now_secs())),
            &inset,
        ),
    ));

    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::MoadimRoutine(routine.title.clone());
    }
    for (key, img) in tiles {
        send_button_image(handle, key, &DynamicImage::ImageRgb8(img));
    }
}
