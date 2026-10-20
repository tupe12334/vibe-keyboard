use crate::domain::actions::{CentyIssue, CentyProject, ClaudeSession};
use crate::domain::moadim::{Flash, MoadimList, Routine};
use crate::domain::profile::SideConfig;

#[derive(Clone)]
pub enum Screen {
    MainPage {
        page: usize,
    },
    CentyProjectList {
        projects: Vec<CentyProject>,
        page: usize,
        filter: Option<String>,
    },
    CentyProjectActions {
        project: CentyProject,
    },
    CentyIssueList {
        issues: Vec<CentyIssue>,
        page: usize,
        project_name: String,
        org: String,
        filter: Option<String>,
    },
    CentyIssueActions {
        issue: CentyIssue,
        project_name: String,
        org: String,
    },
    InputNumber {
        value: String,
    },
    ClaudeSessions {
        sessions: Vec<ClaudeSession>,
        page: usize,
    },
    NewSessionChoice,
    SessionActions {
        session: ClaudeSession,
        page: usize,
    },
    ConfirmExit {
        session: ClaudeSession,
    },
    MoadimList {
        list: MoadimList,
        page: usize,
    },
    MoadimRoutine {
        routine: Routine,
        flash: Option<Flash>,
    },
}

impl Screen {
    /// Side content this screen declares. None keeps the side screen of the home page.
    pub const fn side(&self) -> Option<SideConfig> {
        match self {
            Self::MainPage { .. }
            | Self::CentyProjectList { .. }
            | Self::CentyProjectActions { .. }
            | Self::CentyIssueList { .. }
            | Self::CentyIssueActions { .. }
            | Self::InputNumber { .. }
            | Self::ClaudeSessions { .. }
            | Self::SessionActions { .. }
            | Self::ConfirmExit { .. }
            | Self::MoadimList { .. }
            | Self::MoadimRoutine { .. }
            | Self::NewSessionChoice => None,
        }
    }
}
