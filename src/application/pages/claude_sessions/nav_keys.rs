//! The fixed keys of the sessions page: BACK after the first page, NEW, and MORE or PAGE.

use std::collections::HashMap;

use image::RgbImage;

use crate::domain::actions::ButtonAction;
use crate::domain::profile::KeyInset;
use crate::infrastructure::images::label_image;

/// Adds keys 13 to 15 to `actions` and `tiles`. Key 14 is BACK only past the first page, and key
/// 15 is MORE or PAGE only when there is more than one page.
pub(super) fn add_nav_keys(
    page: usize,
    pages: usize,
    inset: &KeyInset,
    actions: &mut HashMap<u8, ButtonAction>,
    tiles: &mut Vec<(u8, RgbImage)>,
) {
    if page > 0 {
        actions.insert(
            14,
            ButtonAction {
                name: "Back".into(),
                title: "Previous Sessions".into(),
                description: "Previous page of sessions".into(),
            },
        );
        tiles.push((14, label_image("BACK", inset)));
    }
    actions.insert(
        13,
        ButtonAction {
            name: "New".into(),
            title: "New Session".into(),
            description: "Open a new Claude Code window".into(),
        },
    );
    tiles.push((13, label_image("NEW +", inset)));
    if pages > 1 {
        actions.insert(
            15,
            ButtonAction {
                name: "More".into(),
                title: "Next Sessions".into(),
                description: "Next page of sessions".into(),
            },
        );
        let word = if page + 1 < pages { "MORE" } else { "PAGE" };
        let label = format!("{word} {}/{pages}", page + 1);
        tiles.push((15, label_image(&label, inset)));
    }
}
