//! Harness for "an edit keeps the rest of the layout": one editor edit goes through the load and
//! save commands, then the saved file must equal the real fixture except the edited path.
#![allow(clippy::expect_used)]

use serde_json::json;
use tauri::test::get_ipc_response;
use toml::Value;
use vibe_keyboard::domain::profile::Profile;

use crate::command_harness::{env_lock, mock_app, request, seed};

pub const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../tests/fixtures/real_profile.toml"
);

fn fixture_value() -> Value {
    let text = std::fs::read_to_string(FIXTURE).expect("read fixture");
    toml::from_str(&text).expect("parse fixture")
}

/// Removes the value at `path` (table keys or array indexes). A missing path does nothing. A
/// table the removal leaves empty is removed too, so a table the edit created does not count.
fn remove_path(value: &mut Value, path: &[&str]) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    match value {
        Value::Table(table) => {
            if rest.is_empty() {
                table.remove(*first);
                return;
            }
            let emptied = match table.get_mut(*first) {
                Some(child) => {
                    remove_path(child, rest);
                    child.as_table().is_some_and(toml::Table::is_empty)
                }
                None => false,
            };
            if emptied {
                table.remove(*first);
            }
        }
        Value::Array(items) => {
            let Ok(index) = first.parse::<usize>() else {
                return;
            };
            if rest.is_empty() {
                if index < items.len() {
                    items.remove(index);
                }
            } else if let Some(child) = items.get_mut(index) {
                remove_path(child, rest);
            }
        }
        _ => {}
    }
}

/// Runs `edit` on the real layout as the editor does, through `get_profile` and `save_profile`.
/// Then checks the saved file equals the fixture with every `targets` path removed from both.
pub fn assert_untouched(name: &str, edit: impl FnOnce(Profile) -> Profile, targets: &[&[&str]]) {
    let _env = env_lock();
    let path = seed(name);
    std::fs::copy(FIXTURE, &path).expect("place the real layout");
    let (_app, webview) = mock_app();
    let loaded: Profile = get_ipc_response(&webview, request("get_profile", json!({})))
        .expect("load")
        .deserialize()
        .expect("parse layout");
    let edited = edit(loaded);
    get_ipc_response(
        &webview,
        request("save_profile", json!({ "profile": edited, "force": false })),
    )
    .expect("save accepted");
    let saved = std::fs::read_to_string(&path).expect("read saved file");
    let mut after: Value = toml::from_str(&saved).expect("parse saved file");
    let mut before = fixture_value();
    for target in targets {
        remove_path(&mut before, target);
        remove_path(&mut after, target);
    }
    assert_eq!(
        after, before,
        "{name}: a setting the edit did not touch changed"
    );
}
