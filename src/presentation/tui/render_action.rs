//! The name, title and description of a key that has an action.

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use crate::domain::actions::ButtonAction;

pub(super) fn render_action(
    frame: &mut Frame,
    inner: Rect,
    action: &ButtonAction,
    base_style: Style,
) {
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
