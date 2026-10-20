use super::*;

#[test]
fn action_kinds_match_defaults_and_labels() {
    for kind in ActionKind::ALL {
        assert_eq!(Action::default_of(kind).kind(), kind);
        assert!(!kind.label().is_empty());
    }
}
