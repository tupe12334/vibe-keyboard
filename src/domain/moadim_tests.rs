use super::*;

fn routine(running: bool, failures: u32, enabled: bool) -> Routine {
    Routine {
        id: "id".into(),
        title: "t".into(),
        enabled,
        running,
        failures,
        last_trigger: None,
    }
}

#[test]
fn running_wins_over_failed_and_enabled() {
    assert_eq!(routine(true, 3, true).status(), RoutineStatus::Running);
}

#[test]
fn failed_wins_over_enabled_and_disabled() {
    assert_eq!(routine(false, 1, true).status(), RoutineStatus::Failed);
    assert_eq!(routine(false, 1, false).status(), RoutineStatus::Failed);
}

#[test]
fn enabled_and_disabled_when_healthy() {
    assert_eq!(routine(false, 0, true).status(), RoutineStatus::Enabled);
    assert_eq!(routine(false, 0, false).status(), RoutineStatus::Disabled);
}

#[test]
fn every_status_has_a_label() {
    assert_eq!(RoutineStatus::Running.label(), "Running");
    assert_eq!(RoutineStatus::Failed.label(), "Last run failed");
    assert_eq!(RoutineStatus::Enabled.label(), "Enabled");
    assert_eq!(RoutineStatus::Disabled.label(), "Disabled");
}

#[test]
fn trigger_debounce_drops_a_double_tap_per_routine() {
    let mut debounce = TriggerDebounce::default();
    let t0 = Instant::now();
    assert!(debounce.allow("a", t0));
    assert!(!debounce.allow("a", t0 + Duration::from_millis(1_999)));
    assert!(debounce.allow("b", t0 + Duration::from_millis(1_999)));
    assert!(debounce.allow("a", t0 + TRIGGER_DEBOUNCE));
    assert!(!debounce.allow("a", t0 + TRIGGER_DEBOUNCE + Duration::from_millis(10)));
}
