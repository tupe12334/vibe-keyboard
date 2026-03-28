use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph},
    Frame,
};

const SPINNER: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

use crate::domain::actions::CentyState;

use super::app_state::AppState;

pub fn render(state: &AppState, frame: &mut Frame) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // title
        Constraint::Fill(1),   // button grid
    ])
    .split(frame.area());

    render_title(frame, chunks[0], state);
    render_buttons(frame, chunks[1], state);
}

fn render_title(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = match &state.centy_state {
        Some(CentyState::ProjectList { projects, page }) => format!(
            " Centy    Projects ({}-{} of {})    [11] back    [q] quit ",
            page * 10 + 1,
            (page * 10 + 10).min(projects.len()),
            projects.len(),
        ),
        Some(CentyState::ProjectActions { project, .. }) => {
            format!(" Centy    {}    [11] back    [q] quit ", project.name,)
        }
        None => format!(
            " Vibe Keyboard    Page {} / {}    [q] quit ",
            state.current_page + 1,
            state.total_pages,
        ),
    };
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
    let is_loading = state.loading;
    let in_project_actions = matches!(&state.centy_state, Some(CentyState::ProjectActions { .. }));
    let is_nav = key == 11 || (key == 12 && !in_project_actions);

    let base_style = if is_loading {
        Style::default()
    } else if is_pressed {
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

    if is_loading {
        let frame_idx = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.subsec_millis() / 125) as usize % SPINNER.len())
            .unwrap_or(0);
        let vert = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(inner);
        frame.render_widget(
            Paragraph::new(SPINNER[frame_idx])
                .alignment(Alignment::Center)
                .style(base_style),
            vert[1],
        );
        return;
    }

    if is_nav {
        let label = if key == 11 {
            "◄ back"
        } else if state.centy_state.is_some() {
            "next ►"
        } else {
            "fwd ►"
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
