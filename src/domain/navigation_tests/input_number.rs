use super::*;

#[test]
fn back_input_number_pops() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::InputNumber {
        value: String::new(),
    });
    nav.back();
    assert!(matches!(nav.current(), Screen::MainPage { .. }));
}

#[test]
fn input_number_append_and_value() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::InputNumber {
        value: String::new(),
    });
    nav.input_number_append('1');
    nav.input_number_append('2');
    nav.input_number_append('3');
    assert_eq!(nav.input_number_value(), Some("123"));
}

#[test]
fn input_number_backspace() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::InputNumber { value: "42".into() });
    nav.input_number_backspace();
    assert_eq!(nav.input_number_value(), Some("4"));
}

#[test]
fn input_number_backspace_empty_noop() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::InputNumber {
        value: String::new(),
    });
    nav.input_number_backspace();
    assert_eq!(nav.input_number_value(), Some(""));
}

#[test]
fn input_number_clear() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::InputNumber {
        value: "999".into(),
    });
    nav.input_number_clear();
    assert_eq!(nav.input_number_value(), Some(""));
}

#[test]
fn input_number_value_none_when_not_on_screen() {
    let nav = NavigationStack::new(0, 2);
    assert_eq!(nav.input_number_value(), None);
}

#[test]
fn input_number_append_noop_when_not_on_screen() {
    let mut nav = NavigationStack::new(0, 2);
    nav.input_number_append('5'); // no-op
    assert!(matches!(nav.current(), Screen::MainPage { .. }));
}

// --- InputNumber ---
#[test]
fn can_forward_input_number_false() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::InputNumber {
        value: String::new(),
    });
    assert!(!nav.can_forward());
}

#[test]
fn forward_input_number_noop() {
    let mut nav = NavigationStack::new(0, 2);
    nav.push(Screen::InputNumber { value: "42".into() });
    nav.forward();
    assert!(matches!(nav.current(), Screen::InputNumber { .. }));
}
