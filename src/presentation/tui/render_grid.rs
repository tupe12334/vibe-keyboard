//! The button grid below the title: three rows of five keys.

use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use super::app_state::AppState;
use super::render::render_button;

pub(super) fn render_buttons(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let rows = Layout::vertical([
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(7),
    ])
    .split(area);

    for (row_idx, row_area) in rows.iter().enumerate() {
        let cols = Layout::horizontal([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
        ])
        .split(*row_area);

        for (col_idx, col_area) in cols.iter().enumerate() {
            let key_num = (row_idx * 5 + col_idx + 1) as u8;
            render_button(frame, *col_area, key_num, state);
        }
    }
}
