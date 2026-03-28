use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph},
    Frame,
};

use super::app_state::AppState;

pub fn render(state: &AppState, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // title
        Constraint::Fill(1),    // button grid
    ])
    .split(frame.area());

    render_title(frame, chunks[0], state);
    render_buttons(frame, chunks[1], state);
}

fn render_title(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = format!(
        " Vibe Keyboard    Page {} / {}    [q] quit ",
        state.current_page + 1,
        state.total_pages
    );
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::bordered()),
        area,
    );
}

fn render_buttons(frame: &mut Frame, area: Rect, state: &AppState) {
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

fn render_button(frame: &mut Frame, area: Rect, key: u8, state: &AppState) {
    let is_pressed = state.pressed_key == Some(key);
    let is_nav = key == 11 || key == 12;

    let base_style = if is_pressed {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else if is_nav {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let block = Block::bordered()
        .title(format!(" {key} "))
        .style(base_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if is_nav {
        let label = if key == 11 { "◄ back" } else { "fwd ►" };
        let vert = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)])
            .split(inner);
        frame.render_widget(
            Paragraph::new(label).alignment(Alignment::Center).style(base_style),
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

