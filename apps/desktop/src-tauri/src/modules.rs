// Included by main.rs so every module stays at the crate root.
mod accessibility;
mod app;
mod app_commands;
mod bindings;
#[cfg(test)]
mod bundle_info_tests;
#[cfg(test)]
mod command_harness;
mod commands;
mod display_commands;
#[cfg(test)]
mod display_tests;
mod editor_launch;
#[cfg(test)]
mod editor_preserve_hold_tests;
#[cfg(test)]
mod editor_preserve_key_tests;
#[cfg(test)]
mod editor_preserve_layout_tests;
#[cfg(test)]
mod editor_preserve_side_tests;
#[cfg(test)]
mod editor_preserve_startup_tests;
#[cfg(test)]
mod editor_preserve_support;
mod events;
mod key_commands;
mod layout_commands;
mod lifecycle;
mod login_item;
mod profile_io;
#[cfg(test)]
mod profile_roundtrip_tests;
mod prompt_commands;
mod quit;
mod side_commands;
#[cfg(test)]
mod side_tests;
mod status;
mod supervise;
mod thread;
mod tray;
