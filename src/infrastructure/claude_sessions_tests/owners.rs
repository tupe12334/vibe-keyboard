use super::*;
use crate::domain::actions::TerminalApp;

#[test]
fn a_tty_belongs_to_the_app_that_lists_it_and_iterm_wins_a_tie() {
    let map = |pairs: &[(&str, &str)]| -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    };
    let terminal = map(&[("ttys001", "a"), ("ttys002", "b")]);
    let iterm = map(&[("ttys002", "c"), ("ttys003", "d")]);
    let owners = tab_owners(&terminal, &iterm);
    assert_eq!(owners.get("ttys001"), Some(&TerminalApp::Terminal));
    assert_eq!(owners.get("ttys002"), Some(&TerminalApp::ITerm));
    assert_eq!(owners.get("ttys003"), Some(&TerminalApp::ITerm));
    assert_eq!(owners.get("ttys004"), None);
}
