pub use super::button::ButtonAction;
pub use super::claude_session::{
    ClaudeSession, Glyph, NewSessionTerminal, SessionStatus, TerminalApp,
};

#[derive(Clone)]
pub struct CentyProject {
    pub name: String,
    pub org: String,
    pub path: Option<String>,
    pub url: String,
}

#[derive(Clone)]
pub struct CentyIssue {
    pub number: u64,
    pub title: String,
    pub status: String,
    pub id: String,
    pub file_path: Option<String>,
}

pub enum ScreenView {
    MainPage {
        page: usize,
    },
    CentyProjectList {
        total: usize,
        page: usize,
        filter: Option<String>,
    },
    CentyProjectActions {
        project_name: String,
    },
    CentyIssueList {
        total: usize,
        page: usize,
        project_name: String,
        filter: Option<String>,
    },
    CentyIssueActions {
        issue_number: u64,
        project_name: String,
    },
    InputNumber {
        value: String,
    },
    ClaudeSessions {
        total: usize,
        page: usize,
    },
    NewSessionChoice,
    SessionActions {
        label: String,
    },
    MoadimList(usize, usize),
    MoadimRoutine(String),
}
