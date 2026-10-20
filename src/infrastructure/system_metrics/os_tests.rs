use super::*;

/// Prints one live snapshot. Run: `cargo test system_metrics -- --ignored --nocapture`.
#[test]
#[allow(clippy::print_stdout, reason = "manual hardware probe")]
#[ignore = "reads real hardware sensors"]
fn print_one_snapshot() {
    let mut probe = Probe::new();
    thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    let metrics = probe.read();
    let labels: Vec<String> = probe
        .components
        .list()
        .iter()
        .map(|c| format!("{} = {:?}", c.label(), c.temperature()))
        .collect();
    println!("METRICS {metrics:?}");
    println!("SENSORS\n{}", labels.join("\n"));
}
