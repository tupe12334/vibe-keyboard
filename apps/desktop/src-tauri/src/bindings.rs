//! Writes the TypeScript bindings for the frontend, in debug builds only.
use std::path::Path;

use tauri::Wry;
use tauri_specta::{collect_commands, collect_events, Builder};

use crate::{
    app_commands, commands, display_commands, events, key_commands, layout_commands,
    prompt_commands, side_commands,
};

/// The command and event list the frontend binds to. Also the source of the TypeScript bindings.
pub fn specta_builder() -> Builder<Wry> {
    Builder::<Wry>::new()
        .commands(collect_commands![
            commands::get_profile,
            commands::save_profile,
            key_commands::key_preview,
            key_commands::list_builtins,
            key_commands::action_problem,
            side_commands::side_preview,
            commands::device_status,
            layout_commands::editor_rules,
            layout_commands::add_page,
            layout_commands::delete_page,
            prompt_commands::add_prompt,
            prompt_commands::move_prompt,
            prompt_commands::delete_prompt,
            app_commands::list_running_apps,
            app_commands::check_bundle_id,
            display_commands::display_defaults,
        ])
        .events(collect_events![
            events::DeviceConnected,
            events::DeviceDisconnected,
            events::KeyPressed,
            events::ProfileReloaded,
        ])
}

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
