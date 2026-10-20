use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph},
    Frame,
};
use throbber_widgets_tui::Throbber;

use super::app_state::AppState;
use super::render_action::render_action;
use super::render_title;

pub fn render(state: &mut AppState, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // title
        Constraint::Fill(1),   // button grid
    ])
    .split(frame.area());

    render_title::render_title(frame, chunks[0], state);
    render_buttons(frame, chunks[1], state);
}

fn render_buttons(frame: &mut Frame, area: Rect, state: &mut AppState) {
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

fn render_button(frame: &mut Frame, area: Rect, key: u8, state: &mut AppState) {
    let is_pressed = state.pressed_key == Some(key);
    let is_loading = state.loading;
    let is_nav = key == 11 || key == 12 || key == 13;
    let nav_active = match key {
        11 => state.nav_can_back,
        12 => state.nav_can_out,
        13 => state.nav_can_forward,
        _ => false,
    };

    let base_style = if is_loading {
        Style::default()
    } else if is_pressed {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else if is_nav && nav_active {
        Style::default().fg(Color::Cyan)
    } else if is_nav {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM)
    } else {
        Style::default()
    };

    let block = Block::bordered()
        .title(format!(" {key} "))
        .style(base_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if is_loading {
        let vert = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(inner);
        frame.render_stateful_widget(
            Throbber::default().style(base_style),
            vert[1],
            &mut state.throbber_state,
        );
        return;
    }

    if is_nav {
        let label = match key {
            11 => "◄ back",
            12 => "✕ out",
            13 => "fwd ►",
            _ => unreachable!(),
        };
        let vert = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(inner);
        frame.render_widget(
            Paragraph::new(label)
                .alignment(Alignment::Center)
                .style(base_style),
            vert[1],
        );
        return;
    }

    if let Some(action) = state.actions.get(&key) {
        render_action(frame, inner, action, base_style);
    }
}
