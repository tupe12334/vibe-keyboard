//! Writes the TypeScript bindings for the frontend, in debug builds only.
use std::path::Path;

use tauri::Wry;
use tauri_specta::Builder;

/// Write the TypeScript bindings only when their content changed. The dev watcher restarts
/// the app on every write to a watched file, so an unchanged write would loop forever.
#[cfg(debug_assertions)]
pub fn export_bindings(builder: &Builder<Wry>) {
    let target = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/bindings.ts"));
    let staged =
        std::env::temp_dir().join(format!("vibe-keyboard-bindings-{}.ts", std::process::id()));
    builder
        .export(specta_typescript::Typescript::default(), &staged)
        .expect("failed to export TypeScript bindings");
    let generated = std::fs::read(&staged).expect("failed to read generated bindings");
    let _ = std::fs::remove_file(&staged);
    if std::fs::read(target).ok().as_deref() != Some(generated.as_slice()) {
        std::fs::write(target, &generated).expect("failed to write TypeScript bindings");
    }
}
