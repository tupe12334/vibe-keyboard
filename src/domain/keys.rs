/// Keys 11 (back), 12 (out) and 13 (forward) always navigate; profiles cannot bind them.
pub const fn is_nav_key(key: u8) -> bool {
    matches!(key, 11..=13)
}

/// Inverse of `raw_to_logical`: the raw physical key id (0x01–0x0F) for a logical key 1–15.
pub fn logical_to_raw(logical: u8) -> Option<u8> {
    (0x01..=0x0F).find(|&raw| raw_to_logical(raw) == Some(logical))
}

pub const fn raw_to_logical(raw: u8) -> Option<u8> {
    match raw {
        0x0D => Some(1),
        0x0A => Some(2),
        0x07 => Some(3),
        0x04 => Some(4),
        0x01 => Some(5),
        0x0E => Some(6),
        0x0B => Some(7),
        0x08 => Some(8),
        0x05 => Some(9),
        0x02 => Some(10),
        0x0F => Some(11),
        0x0C => Some(12),
        0x09 => Some(13),
        0x06 => Some(14),
        0x03 => Some(15),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_known_mappings() {
        let cases = [
            (0x0D, 1u8),
            (0x0A, 2),
            (0x07, 3),
            (0x04, 4),
            (0x01, 5),
            (0x0E, 6),
            (0x0B, 7),
            (0x08, 8),
            (0x05, 9),
            (0x02, 10),
            (0x0F, 11),
            (0x0C, 12),
            (0x09, 13),
            (0x06, 14),
            (0x03, 15),
        ];
        for (raw, expected) in cases {
            assert_eq!(raw_to_logical(raw), Some(expected), "raw=0x{raw:02X}");
        }
    }

    #[test]
    fn logical_to_raw_round_trips_all_fifteen_keys() {
        for logical in 1..=15u8 {
            let raw = logical_to_raw(logical);
            assert!(raw.is_some(), "logical {logical} has no raw id");
            assert_eq!(raw.and_then(raw_to_logical), Some(logical));
        }
        assert_eq!(logical_to_raw(0), None);
        assert_eq!(logical_to_raw(16), None);
        assert_eq!(logical_to_raw(1), Some(0x0D));
        assert_eq!(logical_to_raw(15), Some(0x03));
    }

    #[test]
    fn test_unknown_raw_returns_none() {
        assert_eq!(raw_to_logical(0x00), None);
        assert_eq!(raw_to_logical(0xFF), None);
        assert_eq!(raw_to_logical(0x10), None);
    }
}
