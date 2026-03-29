pub fn raw_to_logical(raw: u8) -> Option<u8> {
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
    fn test_unknown_raw_returns_none() {
        assert_eq!(raw_to_logical(0x00), None);
        assert_eq!(raw_to_logical(0xFF), None);
        assert_eq!(raw_to_logical(0x10), None);
    }
}
