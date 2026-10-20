use std::sync::{Arc, Mutex};

use crate::infrastructure::usb::Device;
use image::DynamicImage;

use crate::domain::actions::ScreenView;
use crate::infrastructure::images::{
    generate_numpad_backspace_image, generate_numpad_clear_image, generate_numpad_digit_image,
};
use crate::infrastructure::usb::send_button_image;
use crate::presentation::tui;

#[path = "numpad.rs"]
mod numpad;

pub fn render_input_number(value: &str, handle: &Device, state: &Arc<Mutex<tui::AppState>>) {
    let actions = numpad::numpad_buttons();

    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::InputNumber {
            value: value.to_string(),
        };
    }

    send_button_image(
        handle,
        1,
        &DynamicImage::ImageRgb8(generate_numpad_clear_image()),
    );
    send_button_image(
        handle,
        2,
        &DynamicImage::ImageRgb8(generate_numpad_backspace_image()),
    );

    for (key, digit) in [
        (3u8, 1u8),
        (4, 2),
        (5, 3),
        (8, 4),
        (9, 5),
        (10, 6),
        (13, 7),
        (14, 8),
        (15, 9),
    ] {
        send_button_image(
            handle,
            key,
            &DynamicImage::ImageRgb8(generate_numpad_digit_image(digit)),
        );
    }
}
