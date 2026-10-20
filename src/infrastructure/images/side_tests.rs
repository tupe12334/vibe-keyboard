use super::*;

use std::path::PathBuf;

fn all_widgets() -> Vec<Widget> {
    vec![
        Widget::CpuBar,
        Widget::CpuPercent,
        Widget::Temperature,
        Widget::DiskFree,
        Widget::Text {
            value: "BUILD OK".into(),
        },
    ]
}

fn sample() -> Metrics {
    Metrics {
        cpu_percent: 87,
        disk: Some(Disk {
            free: 412_000_000_000,
            total: 994_000_000_000,
        }),
        temperature: Some(71.4),
    }
}

fn black_on_white(widgets: Vec<Widget>) -> SideConfig {
    SideConfig {
        bg: "#000000".into(),
        fg: "#FFFFFF".into(),
        widgets,
        ..SideConfig::default()
    }
}

#[path = "side_tests/canvas.rs"]
mod canvas;

#[path = "side_tests/widget_stack.rs"]
mod widget_stack;

#[path = "side_tests/bar_and_values.rs"]
mod bar_and_values;
