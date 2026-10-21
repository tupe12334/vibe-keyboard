//! The bundle must say why it sends Apple Events. Without `NSAppleEventsUsageDescription`,
//! macOS can refuse `osascript` (-1743) with no prompt when launched from Finder or login.

use std::path::Path;

#[test]
fn bundle_info_plist_declares_apple_events_usage() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let conf = std::fs::read_to_string(dir.join("tauri.conf.json")).unwrap();
    let conf: serde_json::Value = serde_json::from_str(&conf).unwrap();
    let plist = conf["bundle"]["macOS"]["infoPlist"]
        .as_str()
        .expect("bundle.macOS.infoPlist is set");
    let xml = std::fs::read_to_string(dir.join(plist)).expect("infoPlist file exists");
    assert!(
        xml.contains("<key>NSAppleEventsUsageDescription</key>"),
        "{plist} lacks NSAppleEventsUsageDescription"
    );
}
