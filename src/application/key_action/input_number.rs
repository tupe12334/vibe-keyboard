use super::Action;
use crate::domain::navigation::Screen;

/// Keys on the input-number screen: digits, clear and backspace.
pub(super) const fn action(key: u8, screen: &Screen) -> Action {
    match screen {
        Screen::InputNumber { .. } => match key {
            1 => Action::InputNumberClear,
            2 => Action::InputNumberBackspace,
            3 => Action::InputNumberDigit { digit: '1' },
            4 => Action::InputNumberDigit { digit: '2' },
            5 => Action::InputNumberDigit { digit: '3' },
            8 => Action::InputNumberDigit { digit: '4' },
            9 => Action::InputNumberDigit { digit: '5' },
            10 => Action::InputNumberDigit { digit: '6' },
            13 => Action::InputNumberDigit { digit: '7' },
            14 => Action::InputNumberDigit { digit: '8' },
            15 => Action::InputNumberDigit { digit: '9' },
            _ => Action::None,
        },
        _ => Action::None,
    }
}
