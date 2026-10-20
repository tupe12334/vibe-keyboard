#[cfg(test)]
use crate::domain::actions::{CentyIssue, CentyProject, ClaudeSession};
#[cfg(test)]
use crate::domain::moadim::{Flash, MoadimList, Routine};
#[cfg(test)]
use std::time::Instant;

mod back_target;
mod input_digits;
mod paging;
mod screen;
mod stack_state;
pub use back_target::{back_key_on, BackKey};
use paging::moadim_len;
pub use paging::{action_pages, exit_on_page, prompt_index, PROMPTS_PER_PAGE};
pub use screen::Screen;
mod forward;
mod lists;
mod moadim;
mod movement;

/// Which screen is shown: stack depth, screen kind, and its page (0 when it has none).
pub type ScreenId = (usize, std::mem::Discriminant<Screen>, usize);

pub struct NavigationStack {
    stack: Vec<Screen>,
    total_main_pages: usize,
}

#[cfg(test)]
#[path = "navigation_tests/support.rs"]
mod support;

#[cfg(test)]
#[path = "navigation_tests/back_key.rs"]
mod back_key;
#[cfg(test)]
#[path = "navigation_tests/back_pages.rs"]
mod back_pages;
#[cfg(test)]
#[path = "navigation_tests/can_forward.rs"]
mod can_forward;
#[cfg(test)]
#[path = "navigation_tests/forward_actions.rs"]
mod forward_actions;
#[cfg(test)]
#[path = "navigation_tests/forward_pages.rs"]
mod forward_pages;
#[cfg(test)]
#[path = "navigation_tests/input_number.rs"]
mod input_number;
#[cfg(test)]
#[path = "navigation_tests/moadim_flash.rs"]
mod moadim_flash;
#[cfg(test)]
#[path = "navigation_tests/moadim_list.rs"]
mod moadim_list;
#[cfg(test)]
#[path = "navigation_tests/out_and_actions.rs"]
mod out_and_actions;
#[cfg(test)]
#[path = "navigation_tests/screen_id.rs"]
mod screen_id;
#[cfg(test)]
#[path = "navigation_tests/session_actions.rs"]
mod session_actions;
#[cfg(test)]
#[path = "navigation_tests/sessions.rs"]
mod sessions;
#[cfg(test)]
#[path = "navigation_tests/sort.rs"]
mod sort;
#[cfg(test)]
#[path = "navigation_tests/stack.rs"]
mod stack;
