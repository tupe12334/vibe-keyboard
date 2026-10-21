//! The app identity and its permissions are pinned. Changing the identifier makes macOS treat
//! the app as new and drops the user's Accessibility and Automation grants, so push-to-talk and
//! session keys stop. Widening the capabilities lets the editor webview call more APIs.

use std::path::Path;

#[test]
fn bundle_identifier_is_pinned() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let conf = std::fs::read_to_string(dir.join("tauri.conf.json")).unwrap();
    let conf: serde_json::Value = serde_json::from_str(&conf).unwrap();
    assert_eq!(conf["identifier"], "com.tupe12334.vibe-keyboard");
}

#[test]
fn editor_capability_grants_only_core_default() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cap = std::fs::read_to_string(dir.join("capabilities/default.json")).unwrap();
    let cap: serde_json::Value = serde_json::from_str(&cap).unwrap();
    assert_eq!(cap["permissions"], serde_json::json!(["core:default"]));
}
