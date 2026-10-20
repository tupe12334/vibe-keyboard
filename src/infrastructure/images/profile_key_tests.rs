use super::*;
use crate::infrastructure::images::calibration::CALIBRATION_LINE;
use crate::infrastructure::images::calibration_image;

use crate::domain::actions::SessionStatus;

use crate::domain::profile::Profile;

use std::path::PathBuf;

#[path = "profile_key_tests/content_box.rs"]
mod content_box;

#[path = "profile_key_tests/fill_visible_rect.rs"]
mod fill_visible_rect;

#[path = "profile_key_tests/calibration_tiles.rs"]
mod calibration_tiles;
#[path = "profile_key_tests/key_bitmaps.rs"]
mod key_bitmaps;
#[path = "profile_key_tests/key_inset.rs"]
mod key_inset;
