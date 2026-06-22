use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;
use rusb::{Context, DeviceHandle};

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::infrastructure::images::{
    generate_numpad_backspace_image, generate_numpad_clear_image, generate_numpad_digit_image,
};
use crate::infrastructure::usb::send_button_image;
use crate::presentation::tui;

pub fn render_input_number(
    value: &str,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
) {
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();

    actions.insert(
        1,
        ButtonAction {
            name: "CLR".into(),
            title: "Clear".into(),
            description: "Clear all input".into(),
        },
    );
    actions.insert(
        2,
        ButtonAction {
            name: "⌫".into(),
            title: "Backspace".into(),
            description: "Delete last digit".into(),
        },
    );

    // Keys 3–5 → digits 1–3
    for (key, digit) in [(3u8, '1'), (4, '2'), (5, '3')] {
        actions.insert(
            key,
            ButtonAction {
                name: digit.to_string(),
                title: format!("Digit {digit}"),
                description: String::new(),
            },
        );
    }

    // Keys 8–10 → digits 4–6
    for (key, digit) in [(8u8, '4'), (9, '5'), (10, '6')] {
        actions.insert(
            key,
            ButtonAction {
                name: digit.to_string(),
                title: format!("Digit {digit}"),
                description: String::new(),
            },
        );
    }

    // Keys 13–15 → digits 7–9
    for (key, digit) in [(13u8, '7'), (14, '8'), (15, '9')] {
        actions.insert(
            key,
            ButtonAction {
                name: digit.to_string(),
                title: format!("Digit {digit}"),
                description: String::new(),
            },
        );
    }

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
