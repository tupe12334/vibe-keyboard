use super::*;
use crate::infrastructure::images::calibration::CALIBRATION_LINE;
use crate::infrastructure::images::calibration_image;

use crate::domain::actions::SessionStatus;

use crate::domain::profile::Profile;

use std::path::PathBuf;

#[path = "profile_key_tests/part1.rs"]
mod part1;

#[path = "profile_key_tests/part2.rs"]
mod part2;

#[path = "profile_key_tests/part3.rs"]
mod part3;

#[path = "profile_key_tests/content_box.rs"]
mod content_box;

#[path = "profile_key_tests/fill_visible_rect.rs"]
mod fill_visible_rect;
