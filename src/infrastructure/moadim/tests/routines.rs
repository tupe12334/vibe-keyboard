use super::super::*;
use crate::domain::moadim::Routine;

const FIXTURE: &str = include_str!("../list_routines.json");

fn routine(failures: u32, last_trigger: Option<u64>) -> Routine {
    Routine {
        id: "id".into(),
        title: "t".into(),
        enabled: true,
        running: false,
        failures,
        last_trigger,
    }
}

#[test]
fn parses_recorded_list_routines_response() {
    let routines = parse_routines(FIXTURE).unwrap_or_default();
    assert_eq!(routines.len(), 5);
    assert_eq!(routines[0].title, "Sample routine A");
    assert!(routines[0].enabled && routines[0].failures == 0);
    assert!(!routines[1].enabled);
    assert_eq!(routines[2].failures, 31);
    assert_eq!(routines[3].failures, 6);
    assert_eq!(routines[3].last_trigger, Some(1_790_728_218));
    assert_eq!(routines[4].last_trigger, Some(1_792_058_400));
}

#[test]
fn parse_routines_skips_bad_items_and_rejects_non_lists() {
    let body = r#"{"result":{"content":[{"type":"text","text":"[{\"id\":\"a\"},{\"id\":\"b\",\"title\":\"B\",\"enabled\":true,\"is_running\":true,\"consecutive_failures\":-1,\"last_manual_trigger_at\":5,\"last_scheduled_trigger_at\":9}]"}]}}"#;
    let routines = parse_routines(body).unwrap_or_default();
    assert_eq!(routines.len(), 1);
    assert!(routines[0].running);
    assert_eq!(routines[0].failures, 0);
    assert_eq!(routines[0].last_trigger, Some(9));
    assert_eq!(
        parse_routines(r#"{"result":{"content":[{"type":"text","text":"{}"}]}}"#),
        None
    );
    assert_eq!(parse_routines("nope"), None);
}

#[test]
fn ago_picks_minutes_hours_or_days() {
    assert_eq!(ago(None, 100), "NO RUNS");
    assert_eq!(ago(Some(100), 100 + 300), "5M AGO");
    assert_eq!(ago(Some(0), 7_200), "2H AGO");
    assert_eq!(ago(Some(0), 3 * 86_400), "3D AGO");
    assert_eq!(ago(Some(500), 100), "0M AGO");
}

#[test]
fn last_run_label_shows_failures_first() {
    assert_eq!(last_run_label(&routine(4, Some(0)), 7_200), "FAIL x4");
    assert_eq!(last_run_label(&routine(0, Some(0)), 7_200), "2H AGO");
    assert_eq!(last_run_label(&routine(0, None), 7_200), "NO RUNS");
}
