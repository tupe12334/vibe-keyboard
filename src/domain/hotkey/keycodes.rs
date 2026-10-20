//! macOS virtual key codes and CoreGraphics modifier bits for the hotkeys in `hotkey.rs`.

use super::{parse_hotkey, Hotkey, Key, Modifier};

/// The push-to-talk key when a `push_to_talk` action sets none: `Space`, the default key of Claude Code dictation.
pub const fn default_push_to_talk() -> Hotkey {
    Hotkey {
        modifiers: Vec::new(),
        key: Key::Code(49),
    }
}

/// Parse the `key` of a `push_to_talk` action. `None` means the default key.
pub fn push_to_talk_key(text: Option<&str>) -> Result<Hotkey, String> {
    text.map_or_else(|| Ok(default_push_to_talk()), parse_hotkey)
}

/// macOS virtual key code for `key`, or `None` for a character this table does not know.
pub fn key_code(key: &Key) -> Option<u16> {
    match key {
        Key::Code(code) => Some(u16::from(*code)),
        Key::Char(c) => ANSI_CHARS
            .iter()
            .find(|(ch, _)| ch == c)
            .map(|(_, code)| *code),
    }
}

/// ANSI US layout virtual key codes for letters and digits (`kVK_ANSI_*`).
const ANSI_CHARS: [(char, u16); 36] = [
    ('a', 0x00),
    ('s', 0x01),
    ('d', 0x02),
    ('f', 0x03),
    ('h', 0x04),
    ('g', 0x05),
    ('z', 0x06),
    ('x', 0x07),
    ('c', 0x08),
    ('v', 0x09),
    ('b', 0x0B),
    ('q', 0x0C),
    ('w', 0x0D),
    ('e', 0x0E),
    ('r', 0x0F),
    ('y', 0x10),
    ('t', 0x11),
    ('o', 0x1F),
    ('u', 0x20),
    ('i', 0x22),
    ('p', 0x23),
    ('l', 0x25),
    ('j', 0x26),
    ('k', 0x28),
    ('n', 0x2D),
    ('m', 0x2E),
    ('1', 0x12),
    ('2', 0x13),
    ('3', 0x14),
    ('4', 0x15),
    ('5', 0x17),
    ('6', 0x16),
    ('7', 0x1A),
    ('8', 0x1C),
    ('9', 0x19),
    ('0', 0x1D),
];

/// `CGEventFlags` bits for the modifiers of a key press.
pub fn modifier_flags(modifiers: &[Modifier]) -> u64 {
    modifiers.iter().fold(0, |flags, modifier| {
        flags
            | match modifier {
                Modifier::Command => 1 << 20,
                Modifier::Shift => 1 << 17,
                Modifier::Control => 1 << 18,
                Modifier::Option => 1 << 19,
            }
    })
}

#[cfg(test)]
#[path = "keycodes_tests.rs"]
mod tests;
