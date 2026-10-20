use crate::domain::actions::{CentyProject, ClaudeSession};
use crate::domain::navigation::Screen;
use crate::plugins::PluginHost;

/// Records each side effect as a short line, and each shown screen.
#[derive(Default)]
pub(super) struct FakeHost {
    pub calls: Vec<String>,
    pub shown: Vec<Screen>,
    pub projects: Vec<CentyProject>,
}

impl FakeHost {
    fn call(&mut self, what: &str) {
        self.calls.push(what.to_string());
    }
}

impl PluginHost for FakeHost {
    fn open_terminal(&mut self) {
        self.call("terminal");
    }
    fn open_claude_terminal(&mut self) {
        self.call("claude");
    }
    fn open_in_chrome(&mut self, url: &str) {
        self.call(&format!("chrome {url}"));
    }
    fn open_log_file(&mut self) {
        self.call("log");
    }
    fn open_config_in_vscode(&mut self) {
        self.call("config");
    }
    fn open_routines(&mut self) {
        self.call("routines");
    }
    fn fetch_centy_projects(&mut self) -> Vec<CentyProject> {
        self.call("fetch projects");
        self.projects.clone()
    }
    fn list_sessions(&mut self) -> Vec<ClaudeSession> {
        self.call("list sessions");
        Vec::new()
    }
    fn set_loading(&mut self, loading: bool) {
        self.call(&format!("loading {loading}"));
    }
    fn show(&mut self, screen: Screen) {
        self.call("show");
        self.shown.push(screen);
    }
}
