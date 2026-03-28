use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

use super::app_state::AppState;

pub fn render(state: &AppState, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // title
        Constraint::Length(12), // button grid (3 rows × 4 lines)
        Constraint::Min(4),     // event log
    ])
    .split(frame.area());

    render_title(frame, chunks[0], state);
    render_buttons(frame, chunks[1], state);
    render_log(frame, chunks[2], state);
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
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(4),
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

    let label = match key {
        11 => "◄ back".to_string(),
        12 => "fwd ►".to_string(),
        n => format!("{n}"),
    };

    let style = if is_pressed {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else if is_nav {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let block = Block::bordered().style(style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let vert = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .split(inner);

    frame.render_widget(
        Paragraph::new(label)
            .alignment(Alignment::Center)
            .style(style),
        vert[1],
    );
}

fn render_log(frame: &mut Frame, area: Rect, state: &AppState) {
    let lines: Vec<Line> = state.log.iter().map(|s| Line::from(s.as_str())).collect();
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Events ")),
        area,
    );
}
