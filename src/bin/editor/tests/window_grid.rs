use super::*;

#[test]
fn grid_fits_the_default_and_minimum_window() {
    // Window minus both side panels, the top and bottom bars, and the side strip beside the keys.
    let central = |window: Vec2| {
        let height = window.y - 60.0;
        let strip = side_strip_size(height).x + GAP;
        Vec2::new(window.x - PAGES_WIDTH - KEY_PANEL_WIDTH - strip, height)
    };
    for window in [Vec2::new(1200.0, 720.0), Vec2::new(900.0, 600.0)] {
        let area = central(window);
        let cell = cell_size(area);
        let width = 5.0 * cell + 4.0 * GAP + GRID_MARGIN;
        let height = 3.0 * cell + 2.0 * GAP + GRID_MARGIN;
        assert!(width <= area.x + 0.5, "width at {window:?}");
        assert!(height <= area.y + 0.5, "height at {window:?}");
    }
}
