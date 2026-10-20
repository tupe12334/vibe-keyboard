//! The title line above the keys: the screen name and its paging hints.

use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Paragraph},
    Frame,
};

use super::app_state::AppState;
use crate::domain::actions::ScreenView;

pub(super) fn render_title(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = if state.text_input_mode {
        format!(" Filter: {}_", state.text_input_value)
    } else {
        match &state.screen {
            ScreenView::CentyProjectList {
                total,
                page,
                filter,
            } => {
                let filter_str = filter
                    .as_ref()
                    .map(|f| format!("  [filter: \"{f}\"]"))
                    .unwrap_or_default();
                format!(
                " Centy    Projects{filter_str} ({}-{} of {})    [11] out  [12] out  [13] next    [q] quit ",
                page * 10 + 1,
                (page * 10 + 10).min(*total),
                total,
            )
            }
            ScreenView::CentyProjectActions { project_name } => format!(
                " Centy    {project_name}    [11] out  [12] out    [q] quit ",
            ),
            ScreenView::CentyIssueList {
                total,
                page,
                project_name,
                filter,
            } => {
                let filter_str = filter
                    .as_ref()
                    .map(|f| format!("  [filter: \"{f}\"]"))
                    .unwrap_or_default();
                format!(
                " Centy    {} — Issues{filter_str} ({}-{} of {})    [11] out  [12] out  [13] next    [q] quit ",
                project_name,
                page * 10 + 1,
                (page * 10 + 10).min(*total),
                total,
            )
            }
            ScreenView::MainPage { page } => format!(
                " Vibe Keyboard    Page {} / {}    [q] quit ",
                page + 1,
                state.total_pages,
            ),
            ScreenView::CentyIssueActions {
                issue_number,
                project_name,
            } => format!(
                " Centy    {project_name} — Issue #{issue_number}    [11] out  [12] out    [q] quit ",
            ),
            ScreenView::InputNumber { value } => format!(
                " Input Number    {}    [11] out  [12] out    [q] quit ",
                if value.is_empty() { "_" } else { value },
            ),
            ScreenView::SessionActions { label } => format!(
                " {label}    [1-10] prompts  [11] out  [12] back  [14] recap  [15] more  [q] quit ",
            ),
            ScreenView::NewSessionChoice => {
                " New Claude session    [1] iTerm  [2] Terminal    [11] out  [q] quit ".into()
            }
            ScreenView::MoadimList(total, page) => format!(
                " Moadim Routines    ({}-{} of {})    [11] out  [14/15] page  [q] quit ",
                page * 10 + 1,
                (page * 10 + 10).min(*total),
                total,
            ),
            ScreenView::MoadimRoutine(title) => format!(
                " {title}    [1] trigger  [2] enable/disable  [3] snooze  [11] out  [q] quit ",
            ),
            ScreenView::ClaudeSessions { total, page } => format!(
                " Claude Sessions    ({}-{} of {})    [11] out  [12] out  [14/15] page  [q] quit ",
                page * 10 + 1,
                (page * 10 + 10).min(*total),
                total,
            ),
        }
    };
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::bordered()),
        area,
    );
}
