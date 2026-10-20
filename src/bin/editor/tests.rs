use super::{apply_prompt_op, PromptOp};

use vibe_keyboard::domain::profile::Prompt;

use super::{
    cell_size, parse_lsappinfo, side_strip_size, Session, GAP, GRID_MARGIN, KEY_PANEL_WIDTH,
    PAGES_WIDTH, UNDO_WINDOW,
};

use eframe::egui::Vec2;

use std::time::{Duration, Instant};

use vibe_keyboard::domain::profile::{KeyConfig, Profile};

/// A layout with one extra page, so it differs from the default.
fn edited_layout() -> Profile {
    let mut profile = Profile::default_from_legacy();
    profile.add_page("Extra");
    profile
}

#[path = "tests/part1.rs"]
mod part1;

#[path = "tests/part2.rs"]
mod part2;
