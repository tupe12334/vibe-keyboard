pub fn raw_to_logical(raw: u8) -> Option<u8> {
    match raw {
        0x0D => Some(1),  0x0A => Some(2),  0x07 => Some(3),  0x04 => Some(4),  0x01 => Some(5),
        0x0E => Some(6),  0x0B => Some(7),  0x08 => Some(8),  0x05 => Some(9),  0x02 => Some(10),
        0x0F => Some(11), 0x0C => Some(12), 0x09 => Some(13), 0x06 => Some(14), 0x03 => Some(15),
        _ => None,
    }
}
