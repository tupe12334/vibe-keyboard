//! Key events from the deck. Pure: no device access.
use crate::domain::keys::raw_to_logical;

/// Logical key number (1–15) of a key press. None for a release, a short packet or an unknown key.
/// Byte 10 is 1 on key down. Byte 9 is the raw key id.
pub const fn key_press(data: &[u8]) -> Option<u8> {
    if data.len() < 11 || data[10] != 1 {
        return None;
    }
    raw_to_logical(data[9])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(raw: u8, down: u8) -> Vec<u8> {
        let mut data = vec![0; 11];
        data[9] = raw;
        data[10] = down;
        data
    }

    #[test]
    fn key_down_maps_to_logical_key() {
        assert_eq!(key_press(&packet(0x0D, 1)), Some(1));
        assert_eq!(key_press(&packet(0x03, 1)), Some(15));
    }

    #[test]
    fn release_short_or_unknown_is_none() {
        assert_eq!(key_press(&packet(0x0D, 0)), None);
        assert_eq!(key_press(&[0; 10]), None);
        assert_eq!(key_press(&packet(0xFF, 1)), None);
    }
}
