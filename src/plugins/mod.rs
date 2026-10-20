//! Built-in plugins: the named actions a profile binds to a key with `Action::Builtin`.
//!
//! Each plugin is one file here with a unit struct that implements [`Plugin`]. To add one:
//! 1. Create `src/plugins/<name>.rs` with a unit struct and its `impl Plugin`.
//! 2. Declare the module below and add the struct to [`PLUGINS`].
//!
//! A plugin causes side effects only through [`PluginHost`], so tests can run it with a fake.
//! A new kind of side effect is a new `PluginHost` method plus its real impl in `application`.
//!
//! Profiles on disk store [`Plugin::name`], so never rename an existing plugin.

use image::RgbImage;

mod centy_projects;
mod centy_web;
mod claude;
mod claude_sessions;
mod host;
mod log_file;
mod moadim;
mod numpad;
mod terminal;
mod vscode_config;

pub use host::PluginHost;

/// One built-in action.
pub trait Plugin: Sync {
    /// Id stored in `profile.toml`.
    fn name(&self) -> &'static str;
    /// Human name for editors.
    fn label(&self) -> &'static str;
    /// Default key picture. `None` means the key shows its label.
    fn tile(&self) -> Option<RgbImage> {
        None
    }
    /// What a key press does.
    fn run(&self, host: &mut dyn PluginHost);
}

/// Every plugin. The first one is the default for a new `Builtin` action.
pub static PLUGINS: &[&dyn Plugin] = &[
    &centy_projects::CentyProjects,
    &terminal::Terminal,
    &claude::Claude,
    &centy_web::CentyWeb,
    &numpad::Numpad,
    &log_file::LogFile,
    &vscode_config::VscodeConfig,
    &claude_sessions::ClaudeSessions,
    &moadim::Moadim,
];

/// The plugin with id `name`, if any.
pub fn find(name: &str) -> Option<&'static dyn Plugin> {
    PLUGINS.iter().copied().find(|p| p.name() == name)
}

/// Every plugin id, in registry order.
pub fn names() -> impl Iterator<Item = &'static str> {
    PLUGINS.iter().map(|p| p.name())
}

#[cfg(test)]
#[path = "plugins_tests.rs"]
mod tests;
