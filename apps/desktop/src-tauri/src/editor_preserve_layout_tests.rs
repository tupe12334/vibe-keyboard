//! Page and prompt-row edits on the real layout. Each keeps every other setting as it was.
#![allow(clippy::expect_used)]

use crate::editor_preserve_support::assert_untouched;
use crate::layout_commands::{add_page, delete_page};
use crate::prompt_commands::{add_prompt, delete_prompt, move_prompt};

#[test]
fn add_page_keeps_the_existing_pages() {
    assert_untouched("add-page", add_page, &[&["pages", "2"]]);
}

#[test]
fn delete_page_keeps_the_other_pages() {
    assert_untouched(
        "delete-page",
        |p| delete_page(p, 1).expect("delete page 2"),
        &[&["pages", "1"]],
    );
}

#[test]
fn add_prompt_keeps_the_existing_prompts() {
    assert_untouched(
        "add-prompt",
        |p| {
            let mut p = add_prompt(p);
            p.prompts[2].text = "/goal new".into();
            p
        },
        &[&["prompts", "2"]],
    );
}

#[test]
fn move_prompt_keeps_the_rest_of_the_layout() {
    assert_untouched(
        "move-prompt",
        |p| {
            let p = move_prompt(p, 0, true);
            assert_eq!(p.prompts[0].label, "Execute");
            p
        },
        &[&["prompts"]],
    );
}

#[test]
fn delete_prompt_keeps_the_rest_of_the_layout() {
    assert_untouched(
        "delete-prompt",
        |p| {
            let p = delete_prompt(p, 0);
            assert_eq!(p.prompts.len(), 1);
            p
        },
        &[&["prompts"]],
    );
}

#[test]
fn prompt_text_edit_keeps_the_other_prompt() {
    assert_untouched(
        "prompt-text",
        |mut p| {
            p.prompts[0].text = "/goal changed".into();
            p
        },
        &[&["prompts", "0", "text"]],
    );
}
