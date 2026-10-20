use super::*;

#[test]
fn path_lookup_wins_when_tmux_is_on_path() {
    assert_eq!(pick_tmux(|_| true), Some("tmux"));
}

#[test]
fn homebrew_is_tried_before_the_other_fixed_paths() {
    let found = |c: &str| c == "/opt/homebrew/bin/tmux" || c == "/usr/local/bin/tmux";
    assert_eq!(pick_tmux(found), Some("/opt/homebrew/bin/tmux"));
}

#[test]
fn falls_through_to_each_fixed_path_in_order() {
    assert_eq!(
        pick_tmux(|c| c == "/usr/local/bin/tmux"),
        Some("/usr/local/bin/tmux")
    );
    assert_eq!(
        pick_tmux(|c| c == "/opt/local/bin/tmux"),
        Some("/opt/local/bin/tmux")
    );
    assert_eq!(pick_tmux(|c| c == "/usr/bin/tmux"), Some("/usr/bin/tmux"));
}

#[test]
fn none_when_tmux_is_nowhere() {
    assert_eq!(pick_tmux(|_| false), None);
}

#[test]
fn login_path_case_skips_bare_name_and_finds_homebrew() {
    let login = |c: &str| c != "tmux" && c == "/opt/homebrew/bin/tmux";
    assert_eq!(pick_tmux(login), Some("/opt/homebrew/bin/tmux"));
}
