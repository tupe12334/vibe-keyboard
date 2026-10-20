use super::*;

/// Position of each needle in `script`. Fails the test when a needle is missing.
fn positions(script: &str, needles: &[&str]) -> Vec<usize> {
    let found: Vec<usize> = needles.iter().filter_map(|n| script.find(n)).collect();
    assert_eq!(found.len(), needles.len(), "missing step in {needles:?}");
    found
}

#[test]
fn focus_scripts_raise_the_iterm_window_in_order() {
    let [iterm, _] = focus_scripts("ttys003").unwrap_or_default();
    let found = positions(
        &iterm,
        &[
            "tell s to select",
            "tell t to select",
            "tell w to select",
            "set index of w to 1",
            "activate",
        ],
    );
    assert!(found.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn focus_scripts_raise_the_terminal_window_in_order() {
    let [_, terminal] = focus_scripts("ttys003").unwrap_or_default();
    let found = positions(
        &terminal,
        &[
            "set selected of t to true",
            "set frontmost of w to true",
            "set index of w to 1",
            "activate",
        ],
    );
    assert!(found.windows(2).all(|w| w[0] < w[1]));
}
