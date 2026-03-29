use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph},
    Frame,
};
use throbber_widgets_tui::Throbber;

use super::app_state::AppState;
use crate::domain::actions::ScreenView;

pub fn render(state: &mut AppState, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // title
        Constraint::Fill(1),   // button grid
    ])
    .split(frame.area());

    render_title(frame, chunks[0], state);
    render_buttons(frame, chunks[1], state);
}

fn render_title(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = match &state.screen {
        ScreenView::CentyProjectList { total, page } => format!(
            " Centy    Projects ({}-{} of {})    [11] back  [12] out  [13] next    [q] quit ",
            page * 10 + 1,
            (page * 10 + 10).min(*total),
            total,
        ),
        ScreenView::CentyProjectActions { project_name } => format!(
            " Centy    {}    [11] back  [12] out    [q] quit ",
            project_name,
        ),
        ScreenView::CentyIssueList {
            total,
            page,
            project_name,
        } => format!(
            " Centy    {} — Issues ({}-{} of {})    [11] back  [12] out  [13] next    [q] quit ",
            project_name,
            page * 10 + 1,
            (page * 10 + 10).min(*total),
            total,
        ),
        ScreenView::MainPage { page } => format!(
            " Vibe Keyboard    Page {} / {}    [q] quit ",
            page + 1,
            state.total_pages,
        ),
        ScreenView::CentyIssueActions {
            issue_number,
            project_name,
        } => format!(
            " Centy    {} — Issue #{}    [11] back  [12] out    [q] quit ",
            project_name, issue_number,
        ),
        ScreenView::InputNumber { value } => format!(
            " Input Number    {}    [11] back  [12] out    [q] quit ",
            if value.is_empty() { "_" } else { value },
        ),
    };
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::bordered()),
        area,
    );
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
        let vert = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1), // name
            Constraint::Length(1), // title
            Constraint::Length(1), // description
            Constraint::Fill(1),
        ])
        .split(inner);

        frame.render_widget(
            Paragraph::new(action.name.as_str())
                .alignment(Alignment::Center)
                .style(base_style.add_modifier(Modifier::BOLD)),
            vert[1],
        );
        frame.render_widget(
            Paragraph::new(action.title.as_str())
                .alignment(Alignment::Center)
                .style(base_style),
            vert[2],
        );
        frame.render_widget(
            Paragraph::new(action.description.as_str())
                .alignment(Alignment::Center)
                .style(base_style.fg(Color::DarkGray)),
            vert[3],
        );
    }
}
