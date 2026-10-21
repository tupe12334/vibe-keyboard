#![allow(clippy::expect_used)]

use crate::bindings::{render_bindings, specta_builder};

/// The committed bindings must match what the current commands and events generate.
#[test]
fn committed_bindings_match_generated() {
    let generated = render_bindings(&specta_builder());
    let committed = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/bindings.ts"))
        .expect("read committed bindings.ts");
    let generated = String::from_utf8(generated).expect("bindings are UTF-8");
    let committed = String::from_utf8(committed).expect("bindings.ts is UTF-8");
    assert_eq!(
        committed.trim_end_matches('\n'),
        generated.trim_end_matches('\n'),
        "bindings.ts is stale: regenerate with `cargo run -p vibe-keyboard-desktop` (debug build)"
    );
}
