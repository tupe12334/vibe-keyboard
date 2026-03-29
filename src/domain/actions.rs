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
}
