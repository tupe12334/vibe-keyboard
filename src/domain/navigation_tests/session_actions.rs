use super::*;

#[test]
fn prompt_keys_page_through_eight_prompts_each() {
    assert_eq!(prompt_index(0, 1, 3), Some(0));
    assert_eq!(prompt_index(0, 3, 3), Some(2));
    assert_eq!(prompt_index(0, 4, 3), None);
    assert_eq!(prompt_index(0, 10, 20), None);
    assert_eq!(prompt_index(0, 11, 3), None);
    assert_eq!(prompt_index(0, 12, 3), None);
    assert_eq!(prompt_index(1, 1, 12), Some(8));
    assert_eq!(prompt_index(1, 8, 16), Some(15));
    assert_eq!(prompt_index(1, 3, 12), Some(10));
    assert_eq!(prompt_index(1, 5, 12), None);
    assert_eq!(action_pages(0), 1);
    assert_eq!(action_pages(8), 1);
    assert_eq!(action_pages(9), 2);
}

#[test]
fn full_screen_and_blink_keys_are_never_prompt_keys() {
    for page in 0..3 {
        assert_eq!(prompt_index(page, FULL_SCREEN_KEY, 30), None);
        assert_eq!(prompt_index(page, BLINK_KEY, 30), None);
    }
    assert_eq!(BLINK_KEY, 10);
}
