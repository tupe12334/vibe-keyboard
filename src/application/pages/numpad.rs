//! Button map for the number input page: Clear, Backspace and digits 1 to 9.

use std::collections::HashMap;

use crate::domain::actions::ButtonAction;

/// Button actions for the numpad, keyed by device key.
pub fn numpad_buttons() -> HashMap<u8, ButtonAction> {
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
    actions
}
