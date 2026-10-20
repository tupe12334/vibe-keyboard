use crate::domain::actions::{CentyProject, ClaudeSession};
use crate::domain::navigation::Screen;

/// Every side effect a plugin can cause. The controller passes the real one; tests pass a fake.
pub trait PluginHost {
    fn open_terminal(&mut self);
    fn open_claude_terminal(&mut self);
    fn open_in_chrome(&mut self, url: &str);
    fn open_log_file(&mut self);
    fn open_config_in_vscode(&mut self);
    /// Push the Moadim routines page and draw it.
    fn open_routines(&mut self);
    fn fetch_centy_projects(&mut self) -> Vec<CentyProject>;
    fn list_sessions(&mut self) -> Vec<ClaudeSession>;
    /// Show or hide the TUI loading spinner.
    fn set_loading(&mut self, loading: bool);
    /// Push `screen` on the deck and draw it.
    fn show(&mut self, screen: Screen);
}
