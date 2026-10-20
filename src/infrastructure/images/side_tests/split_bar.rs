use super::*;

use super::super::bars::halves;
use super::super::widgets::widget_texts;
use crate::domain::profile::BarStyle;
use crate::infrastructure::system_metrics::Memory;

const MAGENTA: [u8; 3] = [255, 0, 255];

fn metrics(cpu: u8, memory_pct: u8) -> Metrics {
    Metrics {
        cpu_percent: cpu,
        memory: Some(Memory {
            used: u64::from(memory_pct),
            total: 100,
        }),
        ..Metrics::default()
    }
}

fn render(widget: Widget, metrics: &Metrics, fill: Option<&str>) -> RgbImage {
    let side = SideConfig {
        bar_style: fill.map(|f| BarStyle {
            fill: Some(f.into()),
            ..BarStyle::default()
        }),
        ..black_on_white(vec![widget])
    };
    side_image(
        Some(&side),
        &PathBuf::from("/nonexistent"),
        metrics,
        &SideInset::default(),
    )
}

#[test]
fn halves_cover_the_whole_bar() {
    assert_eq!(halves(40), (20, 20));
    assert_eq!(halves(41), (21, 20));
    assert_eq!(halves(1), (1, 0));
    for width in 1..=60 {
        let (left, right) = halves(width);
        assert_eq!(left + right, width);
        assert!(left >= right);
    }
}

#[test]
fn left_half_is_cpu_and_right_half_is_memory() {
    // CPU 30 % is green, memory 90 % is red. Both halves fill from the bottom.
    let img = render(Widget::CpuMemBar, &metrics(30, 90), None);
    let stack = stack_rect(safe_rect(&SideInset::default()));
    let (top, bottom) = bar_span(stack.h, true);
    let (left, right) = (stack.x + 40, stack.x + 60);
    let (y_bottom, y_top) = (stack.y + bottom - 1, stack.y + top);
    assert_eq!(img.get_pixel(left, y_bottom).0, GREEN);
    assert_eq!(img.get_pixel(right, y_bottom).0, RED);
    // The top row is above the 30 % and the 90 % fills, so both halves show track.
    assert_eq!(img.get_pixel(left, y_top).0, TRACK);
    assert_eq!(img.get_pixel(right, y_top).0, TRACK);
}

#[test]
fn a_fill_color_paints_both_halves() {
    let img = render(Widget::CpuMemBar, &metrics(30, 90), Some("#FF00FF"));
    let stack = stack_rect(safe_rect(&SideInset::default()));
    let (_, bottom) = bar_span(stack.h, true);
    let y = stack.y + bottom - 1;
    assert_eq!(img.get_pixel(stack.x + 40, y).0, MAGENTA);
    assert_eq!(img.get_pixel(stack.x + 60, y).0, MAGENTA);
}

#[test]
fn mem_bar_reads_memory_and_the_split_text_has_both_values() {
    let m = metrics(10, 75);
    assert_eq!(widget_texts(&[Widget::MemBar], &m), vec!["75%".to_string()]);
    assert_eq!(
        widget_texts(&[Widget::CpuMemBar], &m),
        vec!["10% 75%".to_string()]
    );
    // Missing memory reads as 0 %.
    assert_eq!(
        widget_texts(&[Widget::MemBar], &Metrics::default()),
        vec!["0%".to_string()]
    );
}
