use eframe::egui::{Color32, Vec2};

use vibe_keyboard::infrastructure::images::{SIDE_HEIGHT, SIDE_WIDTH};

/// Physical layout as seen from the front. Matches `device.md`.
pub(super) const GRID: [[u8; 5]; 3] = [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]];
/// Grid cell size in points. The deck key is square.
pub(super) const CELL_MAX: f32 = 96.0;
pub(super) const CELL_MIN: f32 = 24.0;
/// Gap between grid cells, and the margin kept free around the grid.
pub(super) const GAP: f32 = 8.0;
pub(super) const GRID_MARGIN: f32 = 16.0;
pub(super) const PAGES_WIDTH: f32 = 260.0;
pub(super) const PAGES_WIDTH_RANGE: std::ops::RangeInclusive<f32> = 200.0..=360.0;
pub(super) const KEY_PANEL_WIDTH: f32 = 320.0;
pub(super) const KEY_PANEL_WIDTH_RANGE: std::ops::RangeInclusive<f32> = 260.0..=420.0;
pub(super) const RED: Color32 = Color32::from_rgb(230, 80, 80);
/// Side strip preview: at most the device render height, same 100:480 ratio.
pub(super) const SIDE_STRIP_MAX: f32 = SIDE_HEIGHT as f32;

/// Size of the side strip for `avail_height`: at most the device render height, 100:480 ratio.
pub(super) fn side_strip_size(avail_height: f32) -> Vec2 {
    let height = (avail_height - GRID_MARGIN).clamp(1.0, SIDE_STRIP_MAX);
    Vec2::new(height * SIDE_WIDTH as f32 / SIDE_HEIGHT as f32, height)
}

/// Largest square cell that fits the 3x5 grid in `avail`, never below `CELL_MIN`.
pub(super) fn cell_size(avail: Vec2) -> f32 {
    let by_width = (avail.x - GRID_MARGIN - 4.0 * GAP) / 5.0;
    let by_height = (avail.y - GRID_MARGIN - 2.0 * GAP) / 3.0;
    by_width.min(by_height).clamp(CELL_MIN, CELL_MAX)
}
