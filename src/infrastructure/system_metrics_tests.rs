use super::*;

fn disk(free: u64) -> Disk {
    Disk { free, total: 1000 }
}

#[test]
fn percent_is_rounded_and_clamped() {
    assert_eq!(clamp_percent(-3.0), 0);
    assert_eq!(clamp_percent(f32::NAN), 0);
    assert_eq!(clamp_percent(42.4), 42);
    assert_eq!(clamp_percent(42.6), 43);
    assert_eq!(clamp_percent(120.0), 100);
}

#[test]
fn only_root_and_data_volumes_are_candidates() {
    assert!(is_candidate_mount("/"));
    assert!(is_candidate_mount("/System/Volumes/Data"));
    assert!(!is_candidate_mount("/Volumes/Backup"));
    assert!(!is_candidate_mount("/System/Volumes/Preboot"));
}

#[test]
fn disk_choice_takes_the_most_free_space() {
    assert_eq!(choose_disk(&[]), None);
    assert_eq!(choose_disk(&[disk(10), disk(90), disk(40)]), Some(disk(90)));
}

#[test]
fn hottest_cpu_reads_the_maximum_or_none() {
    assert_eq!(hottest_cpu(&[]), None);
    let readings = [
        ("PMU tdie1".to_string(), 71.4),
        ("PMU2 tdie3".to_string(), 51.5),
        ("PMU tdev6".to_string(), 99.0),
        ("NAND CH0 temp".to_string(), 80.0),
        ("PMU tdev4".to_string(), -9201.1),
    ];
    assert_eq!(hottest_cpu(&readings), Some(71.4));
    assert!(is_cpu_sensor("Tdie"));
    assert!(!is_cpu_sensor("PMU tcal"));
}

#[test]
fn build_filters_volumes_and_keeps_missing_values_empty() {
    let disks = [
        ("/Volumes/Backup".to_string(), disk(999)),
        ("/".to_string(), disk(10)),
        ("/System/Volumes/Data".to_string(), disk(30)),
    ];
    let metrics = build(150.0, None, &disks, &[]);
    assert_eq!(metrics.cpu_percent, 100);
    assert_eq!(metrics.disk, Some(disk(30)));
    assert_eq!(metrics.temperature, None);
    assert_eq!(build(0.0, None, &[], &[]), Metrics::default());
}

#[test]
fn snapshot_is_default_before_the_first_sample() {
    let shared: Shared = Arc::new(Mutex::new(None));
    assert_eq!(snapshot(&shared), Metrics::default());
    *shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Metrics {
        cpu_percent: 7,
        ..Metrics::default()
    });
    assert_eq!(snapshot(&shared).cpu_percent, 7);
}

#[test]
fn memory_percent_rounds_and_handles_missing_memory() {
    use super::memory::{memory_percent, Memory};
    let mem = |used, total| Some(Memory { used, total });
    assert_eq!(memory_percent(None), 0);
    assert_eq!(memory_percent(mem(5, 0)), 0);
    assert_eq!(memory_percent(mem(0, 100)), 0);
    assert_eq!(memory_percent(mem(50, 100)), 50);
    assert_eq!(memory_percent(mem(994, 1000)), 99);
    assert_eq!(memory_percent(mem(996, 1000)), 100);
    assert_eq!(memory_percent(mem(2000, 1000)), 100);
}
