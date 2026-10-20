use crate::domain::actions::NewSessionTerminal;

pub(in crate::application) enum Action {
    SelectProject {
        projects: Vec<crate::domain::actions::CentyProject>,
        idx: usize,
    },
    OpenVsCode {
        name: String,
        path: Option<String>,
    },
    OpenTerminalInPath {
        name: String,
        path: Option<String>,
    },
    OpenBrowser {
        name: String,
        url: String,
    },
    FetchIssues {
        project_name: String,
        org: String,
    },
    SelectIssue {
        issues: Vec<crate::domain::actions::CentyIssue>,
        project_name: String,
        org: String,
        idx: usize,
    },
    OpenIssueInVsCode {
        file_path: Option<String>,
    },
    OpenIssueInWeb {
        id: String,
        org: String,
        project_name: String,
    },
    OpenCentyWorkspace {
        issue_number: u64,
    },
    SortList,
    OpenNewSessionChoice,
    OpenSessionActions {
        session: crate::domain::actions::ClaudeSession,
    },
    OpenConfirmExit {
        session: crate::domain::actions::ClaudeSession,
    },
    SendPrompt {
        pid: u32,
        tty: String,
        text: String,
    },
    PromptUnsupported {
        key: u8,
    },
    TurnActionPage {
        forward: bool,
        count: usize,
    },

    NewSession {
        terminal: NewSessionTerminal,
    },
    TurnSessionsPage {
        forward: bool,
    },
    InputNumberDigit {
        digit: char,
    },
    InputNumberClear,
    InputNumberBackspace,
    Search,
    None,
}
