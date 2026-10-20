use super::*;
use crate::domain::hotkey::{Key, Modifier};

#[test]
fn default_push_to_talk_is_space() {
    assert_eq!(push_to_talk_key(None), Ok(default_push_to_talk()));
    assert_eq!(default_push_to_talk().key, Key::Code(49));
    assert_eq!(
        push_to_talk_key(Some("cmd+k")).map(|h| h.key),
        Ok(Key::Char('k'))
    );
    assert!(push_to_talk_key(Some("hyper+k")).is_err());
}

#[test]
fn maps_named_keys_and_letters_to_virtual_codes() {
    assert_eq!(key_code(&Key::Code(49)), Some(49));
    assert_eq!(key_code(&Key::Char('a')), Some(0));
    assert_eq!(key_code(&Key::Char('k')), Some(40));
    assert_eq!(key_code(&Key::Char('0')), Some(29));
    assert_eq!(key_code(&Key::Char('!')), None);
}

#[test]
fn maps_modifiers_to_cg_flag_bits() {
    assert_eq!(modifier_flags(&[]), 0);
    assert_eq!(
        modifier_flags(&[
            Modifier::Command,
            Modifier::Shift,
            Modifier::Control,
            Modifier::Option
        ]),
        (1 << 20) | (1 << 17) | (1 << 18) | (1 << 19)
    );
}
