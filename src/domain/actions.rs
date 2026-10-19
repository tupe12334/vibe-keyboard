use std::time::Duration;

pub struct ButtonAction {
    pub name: String,
    pub title: String,
    pub description: String,
}

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
}

/// Activity of a Claude Code session. Green is `Active`, orange is `Stale`. Declared in sort order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionStatus {
    /// Output on the tty within the last minute, or a spinner on the tab title.
    Active,
    /// No output for over a minute.
    Stale,
}

/// The status glyph Claude Code puts at the start of a terminal tab title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// `✳`: idle, waiting for the user.
    Waiting,
    /// A spinner glyph: working.
    Spinner,
}

/// A Claude Code process on a terminal. `tty` is the bare name, e.g. `ttys003`. `age` is the time
/// since the tty last printed, or `None` when the tty could not be read.
#[derive(Clone, Debug, PartialEq)]
pub struct ClaudeSession {
    pub tty: String,
    pub label: String,
    pub status: SessionStatus,
    pub age: Option<Duration>,
}
