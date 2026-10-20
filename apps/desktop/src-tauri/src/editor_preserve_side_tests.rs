//! Side screen, display insets and page name or app edits on the real layout.
#![allow(clippy::expect_used)]

use crate::editor_preserve_support::assert_untouched;
use vibe_keyboard::domain::profile::{BarStyle, SideConfig, Widget};

#[test]
fn new_side_screen_keeps_the_other_pages() {
    assert_untouched(
        "side-new",
        |mut p| {
            p.set_side(1, Some(SideConfig::default()));
            p
        },
        &[&["pages", "1", "side"]],
    );
}

#[test]
fn side_widget_edit_keeps_the_side_label_and_colors() {
    assert_untouched(
        "side-widgets",
        |mut p| {
            let side = p.pages[0].side.as_mut().expect("side screen");
            side.widgets.push(Widget::Text { value: "hi".into() });
            p
        },
        &[&["pages", "0", "side", "widgets"]],
    );
}

#[test]
fn side_bar_style_edit_keeps_the_side_widgets() {
    assert_untouched(
        "side-bar",
        |mut p| {
            let side = p.pages[0].side.as_mut().expect("side screen");
            side.bar_style = Some(BarStyle {
                thickness: Some(30),
                ..BarStyle::default()
            });
            p
        },
        &[&["pages", "0", "side", "bar_style"]],
    );
}

#[test]
fn key_inset_edit_keeps_the_side_inset() {
    assert_untouched(
        "key-inset",
        |mut p| {
            p.display.key_inset.top = 20;
            p
        },
        &[&["display", "key_inset", "top"]],
    );
}

#[test]
fn side_inset_edit_keeps_the_key_inset() {
    assert_untouched(
        "side-inset",
        |mut p| {
            p.display.side_inset.top = 50;
            p
        },
        &[&["display", "side_inset", "top"]],
    );
}

#[test]
fn page_rename_keeps_the_page_keys() {
    assert_untouched(
        "page-rename",
        |mut p| {
            p.pages[1].name = "Utils".into();
            p
        },
        &[&["pages", "1", "name"]],
    );
}

#[test]
fn page_app_list_edit_keeps_the_page_keys() {
    assert_untouched(
        "page-apps",
        |mut p| {
            p.pages[0].apps = vec!["com.apple.Terminal".into()];
            p
        },
        &[&["pages", "0", "apps"]],
    );
}
