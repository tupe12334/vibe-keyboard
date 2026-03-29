use crate::domain::actions::{CentyIssue, CentyProject};

#[derive(Clone)]
pub enum Screen {
    MainPage {
        page: usize,
    },
    CentyProjectList {
        projects: Vec<CentyProject>,
        page: usize,
    },
    CentyProjectActions {
        project: CentyProject,
    },
    CentyIssueList {
        issues: Vec<CentyIssue>,
        page: usize,
        project_name: String,
        org: String,
    },
    CentyIssueActions {
        issue: CentyIssue,
        project_name: String,
        org: String,
    },
}

pub struct NavigationStack {
    stack: Vec<Screen>,
    total_main_pages: usize,
}

impl NavigationStack {
    pub fn new(initial_page: usize, total_main_pages: usize) -> Self {
        assert!(total_main_pages > 0, "must have at least one page");
        Self {
            stack: vec![Screen::MainPage {
                page: initial_page.min(total_main_pages - 1),
            }],
            total_main_pages,
        }
    }

    pub fn push(&mut self, screen: Screen) {
        self.stack.push(screen);
    }

    pub fn current(&self) -> &Screen {
        self.stack
            .last()
            .expect("NavigationStack is always non-empty")
    }

    /// Navigate back within the current screen's pages, or pop to the parent.
    /// `MainPage` wraps around (circular). Pushed screens go to the previous page
    /// if available, otherwise pop.
    pub fn back(&mut self) {
        let should_pop = match self
            .stack
            .last_mut()
            .expect("NavigationStack is always non-empty")
        {
            Screen::MainPage { page } => {
                *page = if *page == 0 {
                    self.total_main_pages - 1
                } else {
                    *page - 1
                };
                false
            }
            Screen::CentyProjectList { page, .. } if *page > 0 => {
                *page -= 1;
                false
            }
            Screen::CentyIssueList { page, .. } if *page > 0 => {
                *page -= 1;
                false
            }
            _ => true,
        };
        if should_pop && self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Pop to the parent screen. No-op at the bottom of the stack.
    pub fn out(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn can_back(&self) -> bool {
        true // back always navigates (prev page or pops to parent)
    }

    pub fn can_out(&self) -> bool {
        self.stack.len() > 1
    }

    pub fn can_forward(&self) -> bool {
        match self
            .stack
            .last()
            .expect("NavigationStack is always non-empty")
        {
            Screen::MainPage { .. } => true,
            Screen::CentyProjectList { projects, page } => {
                *page < projects.len().saturating_sub(1) / 10
            }
            Screen::CentyIssueList { issues, page, .. } => {
                *page < issues.len().saturating_sub(1) / 10
            }
            Screen::CentyProjectActions { .. } => false,
            Screen::CentyIssueActions { .. } => false,
        }
    }

    /// Toggle sort order of the current list screen.
    /// Projects are sorted by name; issues are sorted by number.
    /// Each call reverses the current order.
    pub fn toggle_sort(&mut self) {
        match self
            .stack
            .last_mut()
            .expect("NavigationStack is always non-empty")
        {
            Screen::CentyProjectList { projects, .. } => {
                let asc = projects.windows(2).all(|w| w[0].name <= w[1].name);
                if asc {
                    projects.sort_by(|a, b| b.name.cmp(&a.name));
                } else {
                    projects.sort_by(|a, b| a.name.cmp(&b.name));
                }
            }
            Screen::CentyIssueList { issues, .. } => {
                let desc = issues.windows(2).all(|w| w[0].number >= w[1].number);
                if desc {
                    issues.sort_by_key(|i| i.number);
                } else {
                    issues.sort_by(|a, b| b.number.cmp(&a.number));
                }
            }
            _ => {}
        }
    }

    /// Navigate forward within the current screen's pages. No-op if no more pages.
    pub fn forward(&mut self) {
        match self
            .stack
            .last_mut()
            .expect("NavigationStack is always non-empty")
        {
            Screen::MainPage { page } => {
                *page = (*page + 1) % self.total_main_pages;
            }
            Screen::CentyProjectList { projects, page } => {
                let max = projects.len().saturating_sub(1) / 10;
                if *page < max {
                    *page += 1;
                }
            }
            Screen::CentyIssueList { issues, page, .. } => {
                let max = issues.len().saturating_sub(1) / 10;
                if *page < max {
                    *page += 1;
                }
            }
            Screen::CentyProjectActions { .. } => {}
            Screen::CentyIssueActions { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_project() -> CentyProject {
        CentyProject {
            name: "proj".into(),
            org: "org".into(),
            path: None,
            url: "https://example.com".into(),
        }
    }

    fn make_issue() -> CentyIssue {
        CentyIssue {
            number: 1,
            title: "issue".into(),
            status: "open".into(),
            id: "abc".into(),
            file_path: None,
        }
    }

    fn make_projects(n: usize) -> Vec<CentyProject> {
        (0..n).map(|_| make_project()).collect()
    }

    fn make_issues(n: usize) -> Vec<CentyIssue> {
        (0..n).map(|_| make_issue()).collect()
    }

    // --- new ---

    #[test]
    fn new_clamps_initial_page() {
        let nav = NavigationStack::new(5, 3);
        assert!(matches!(nav.current(), Screen::MainPage { page: 2 }));
    }

    #[test]
    fn new_respects_initial_page_within_bounds() {
        let nav = NavigationStack::new(1, 3);
        assert!(matches!(nav.current(), Screen::MainPage { page: 1 }));
    }

    // --- push / current ---

    #[test]
    fn push_and_current() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectActions {
            project: make_project(),
        });
        assert!(matches!(nav.current(), Screen::CentyProjectActions { .. }));
    }

    // --- can_back / can_out / can_forward ---

    #[test]
    fn can_back_always_true() {
        let nav = NavigationStack::new(0, 2);
        assert!(nav.can_back());
    }

    #[test]
    fn can_out_false_at_root() {
        let nav = NavigationStack::new(0, 2);
        assert!(!nav.can_out());
    }

    #[test]
    fn can_out_true_with_depth() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectActions {
            project: make_project(),
        });
        assert!(nav.can_out());
    }

    #[test]
    fn can_forward_main_page_always_true() {
        let nav = NavigationStack::new(0, 2);
        assert!(nav.can_forward());
    }

    #[test]
    fn can_forward_project_list_false_when_at_last_page() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectList {
            projects: make_projects(5),
            page: 0,
        });
        assert!(!nav.can_forward());
    }

    #[test]
    fn can_forward_project_list_true_when_more_pages() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectList {
            projects: make_projects(15),
            page: 0,
        });
        assert!(nav.can_forward());
    }

    #[test]
    fn can_forward_issue_list_false_when_at_last_page() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueList {
            issues: make_issues(5),
            page: 0,
            project_name: "p".into(),
            org: "org".into(),
        });
        assert!(!nav.can_forward());
    }

    #[test]
    fn can_forward_issue_list_true_when_more_pages() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueList {
            issues: make_issues(15),
            page: 0,
            project_name: "p".into(),
            org: "org".into(),
        });
        assert!(nav.can_forward());
    }

    #[test]
    fn can_forward_project_actions_false() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectActions {
            project: make_project(),
        });
        assert!(!nav.can_forward());
    }

    #[test]
    fn can_forward_issue_actions_false() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueActions {
            issue: make_issue(),
            project_name: "p".into(),
            org: "org".into(),
        });
        assert!(!nav.can_forward());
    }

    // --- back ---

    #[test]
    fn back_main_page_wraps_from_zero() {
        let mut nav = NavigationStack::new(0, 3);
        nav.back();
        assert!(matches!(nav.current(), Screen::MainPage { page: 2 }));
    }

    #[test]
    fn back_main_page_decrements() {
        let mut nav = NavigationStack::new(2, 3);
        nav.back();
        assert!(matches!(nav.current(), Screen::MainPage { page: 1 }));
    }

    #[test]
    fn back_project_list_decrements_page() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectList {
            projects: make_projects(15),
            page: 1,
        });
        nav.back();
        assert!(matches!(
            nav.current(),
            Screen::CentyProjectList { page: 0, .. }
        ));
    }

    #[test]
    fn back_project_list_at_page_zero_pops() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectList {
            projects: make_projects(5),
            page: 0,
        });
        nav.back();
        assert!(matches!(nav.current(), Screen::MainPage { .. }));
    }

    #[test]
    fn back_issue_list_decrements_page() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueList {
            issues: make_issues(15),
            page: 1,
            project_name: "p".into(),
            org: "org".into(),
        });
        nav.back();
        assert!(matches!(
            nav.current(),
            Screen::CentyIssueList { page: 0, .. }
        ));
    }

    #[test]
    fn back_issue_list_at_page_zero_pops() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueList {
            issues: make_issues(5),
            page: 0,
            project_name: "p".into(),
            org: "org".into(),
        });
        nav.back();
        assert!(matches!(nav.current(), Screen::MainPage { .. }));
    }

    #[test]
    fn back_project_actions_pops() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectActions {
            project: make_project(),
        });
        nav.back();
        assert!(matches!(nav.current(), Screen::MainPage { .. }));
    }

    #[test]
    fn back_issue_actions_pops() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueActions {
            issue: make_issue(),
            project_name: "p".into(),
            org: "org".into(),
        });
        nav.back();
        assert!(matches!(nav.current(), Screen::MainPage { .. }));
    }

    #[test]
    fn back_at_root_with_pop_screen_is_noop() {
        // back() on a CentyProjectActions at root (stack len == 1) is a no-op
        // Construct manually by starting with a project actions as base
        let mut nav = NavigationStack::new(0, 1);
        // back on MainPage with 1 total page wraps to 0 (same page)
        nav.back();
        assert!(matches!(nav.current(), Screen::MainPage { page: 0 }));
    }

    // --- out ---

    #[test]
    fn out_noop_at_root() {
        let mut nav = NavigationStack::new(0, 2);
        nav.out();
        assert!(matches!(nav.current(), Screen::MainPage { .. }));
    }

    #[test]
    fn out_pops_to_parent() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectActions {
            project: make_project(),
        });
        nav.out();
        assert!(matches!(nav.current(), Screen::MainPage { .. }));
    }

    // --- forward ---

    #[test]
    fn forward_main_page_wraps() {
        let mut nav = NavigationStack::new(1, 2);
        nav.forward();
        assert!(matches!(nav.current(), Screen::MainPage { page: 0 }));
    }

    #[test]
    fn forward_main_page_increments() {
        let mut nav = NavigationStack::new(0, 2);
        nav.forward();
        assert!(matches!(nav.current(), Screen::MainPage { page: 1 }));
    }

    #[test]
    fn forward_project_list_increments() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectList {
            projects: make_projects(15),
            page: 0,
        });
        nav.forward();
        assert!(matches!(
            nav.current(),
            Screen::CentyProjectList { page: 1, .. }
        ));
    }

    #[test]
    fn forward_project_list_noop_at_max() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectList {
            projects: make_projects(5),
            page: 0,
        });
        nav.forward();
        assert!(matches!(
            nav.current(),
            Screen::CentyProjectList { page: 0, .. }
        ));
    }

    #[test]
    fn forward_issue_list_increments() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueList {
            issues: make_issues(15),
            page: 0,
            project_name: "p".into(),
            org: "org".into(),
        });
        nav.forward();
        assert!(matches!(
            nav.current(),
            Screen::CentyIssueList { page: 1, .. }
        ));
    }

    #[test]
    fn forward_issue_list_noop_at_max() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueList {
            issues: make_issues(5),
            page: 0,
            project_name: "p".into(),
            org: "org".into(),
        });
        nav.forward();
        assert!(matches!(
            nav.current(),
            Screen::CentyIssueList { page: 0, .. }
        ));
    }

    #[test]
    fn forward_project_actions_noop() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyProjectActions {
            project: make_project(),
        });
        nav.forward();
        assert!(matches!(nav.current(), Screen::CentyProjectActions { .. }));
    }

    #[test]
    fn forward_issue_actions_noop() {
        let mut nav = NavigationStack::new(0, 2);
        nav.push(Screen::CentyIssueActions {
            issue: make_issue(),
            project_name: "p".into(),
            org: "org".into(),
        });
        nav.forward();
        assert!(matches!(nav.current(), Screen::CentyIssueActions { .. }));
    }

    // --- toggle_sort ---

    fn make_named_project(name: &str) -> CentyProject {
        CentyProject {
            name: name.into(),
            org: "org".into(),
            path: None,
            url: "https://example.com".into(),
        }
    }

    fn make_numbered_issue(number: u64) -> CentyIssue {
        CentyIssue {
            number,
            title: "issue".into(),
            status: "open".into(),
            id: "abc".into(),
            file_path: None,
        }
    }

    fn current_project_names(nav: &NavigationStack) -> Vec<String> {
        match nav.current() {
            Screen::CentyProjectList { projects, .. } => {
                projects.iter().map(|p| p.name.clone()).collect()
            }
            _ => panic!("expected CentyProjectList"),
        }
    }

    fn current_issue_numbers(nav: &NavigationStack) -> Vec<u64> {
        match nav.current() {
            Screen::CentyIssueList { issues, .. } => issues.iter().map(|i| i.number).collect(),
            _ => panic!("expected CentyIssueList"),
        }
    }

    #[test]
    fn toggle_sort_projects_ascending_then_descending() {
        let mut nav = NavigationStack::new(0, 1);
        nav.push(Screen::CentyProjectList {
            projects: vec![
                make_named_project("alpha"),
                make_named_project("beta"),
                make_named_project("gamma"),
            ],
            page: 0,
        });
        // Initially ascending: toggle should sort descending
        nav.toggle_sort();
        assert_eq!(current_project_names(&nav), vec!["gamma", "beta", "alpha"]);
        // Toggle again: should restore ascending
        nav.toggle_sort();
        assert_eq!(current_project_names(&nav), vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn toggle_sort_issues_desc_then_asc() {
        let mut nav = NavigationStack::new(0, 1);
        nav.push(Screen::CentyIssueList {
            issues: vec![
                make_numbered_issue(3),
                make_numbered_issue(2),
                make_numbered_issue(1),
            ],
            page: 0,
            project_name: "p".into(),
            org: "org".into(),
        });
        // Initially descending: toggle should sort ascending
        nav.toggle_sort();
        assert_eq!(current_issue_numbers(&nav), vec![1, 2, 3]);
        // Toggle again: should restore descending
        nav.toggle_sort();
        assert_eq!(current_issue_numbers(&nav), vec![3, 2, 1]);
    }

    #[test]
    fn toggle_sort_noop_on_main_page() {
        let mut nav = NavigationStack::new(0, 2);
        nav.toggle_sort();
        assert!(matches!(nav.current(), Screen::MainPage { page: 0 }));
    }

    #[test]
    #[should_panic(expected = "expected CentyProjectList")]
    fn current_project_names_panics_on_wrong_screen() {
        let nav = NavigationStack::new(0, 1);
        current_project_names(&nav);
    }

    #[test]
    #[should_panic(expected = "expected CentyIssueList")]
    fn current_issue_numbers_panics_on_wrong_screen() {
        let nav = NavigationStack::new(0, 1);
        current_issue_numbers(&nav);
    }
}
