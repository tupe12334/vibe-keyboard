use eframe::egui::Vec2;

use vibe_keyboard::infrastructure::images::{SIDE_HEIGHT, SIDE_WIDTH};

use super::{CELL_MAX, CELL_MIN, GAP, GRID_MARGIN, SIDE_STRIP_MAX};

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
