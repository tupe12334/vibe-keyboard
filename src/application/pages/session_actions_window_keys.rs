//! The window keys on a session action page: FULL SCREEN (9) and BLINK (10). Both focus the session
//! first. Focus works for every app.

use image::RgbImage;

use crate::domain::actions::ButtonAction;
use crate::domain::navigation::{BLINK_KEY, FULL_SCREEN_KEY};
use crate::domain::profile::KeyInset;
use crate::infrastructure::images::label_image;

/// Labels on the FULL SCREEN key (9) and the BLINK key (10).
const FULL_SCREEN_LABEL: &str = "FULL SCREEN";
const BLINK_LABEL: &str = "BLINK";

/// The FULL SCREEN key (9) and the BLINK key (10): each tile with its action.
pub fn window_keys(inset: &KeyInset) -> [(u8, RgbImage, ButtonAction); 2] {
    [
        (
            FULL_SCREEN_KEY,
            label_image(FULL_SCREEN_LABEL, inset),
            ButtonAction {
                name: "Full screen".into(),
                title: "Full screen".into(),
                description: "Focus this session and toggle full screen on its window".into(),
            },
        ),
        (
            BLINK_KEY,
            label_image(BLINK_LABEL, inset),
            ButtonAction {
                name: "Blink".into(),
                title: "Blink".into(),
                description: "Focus this session and show a fireworks highlight".into(),
            },
        ),
    ]
}
