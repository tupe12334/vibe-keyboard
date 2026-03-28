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

pub enum CentyState {
    ProjectList {
        projects: Vec<CentyProject>,
        page: usize,
    },
    ProjectActions {
        project: CentyProject,
    },
}
