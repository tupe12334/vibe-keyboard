//! The Moadim routines list page: one tile per routine, with paging.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::domain::moadim::MoadimList;
use crate::infrastructure::images::{label_image, routine_image};
use crate::infrastructure::usb::{send_button_image, Device};
use crate::presentation::tui;

/// Keys 1..=10 show routines, one tile each. Key 14 is BACK after the first page. Key 15 shows
/// `MORE` or `PAGE` with the page counter when there is more than one page.
pub fn render_moadim_list(
    list: &MoadimList,
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
) {
    const PER_PAGE: usize = 10;
    let inset = super::super::key_tiles::key_inset(state);
    let total = match list {
        MoadimList::Online(routines) => routines.len(),
        MoadimList::Pending | MoadimList::Offline => 0,
    };
    let pages = total.div_ceil(PER_PAGE).max(1);
    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    let mut tiles = Vec::new();

    match list {
        MoadimList::Pending => tiles.push((1, label_image("LOADING", &inset))),
        MoadimList::Offline => tiles.push((1, label_image("MOADIM OFFLINE", &inset))),
        MoadimList::Online(routines) => {
            for (i, routine) in routines
                .iter()
                .skip(page * PER_PAGE)
                .take(PER_PAGE)
                .enumerate()
            {
                let key = (i + 1) as u8;
                let status = routine.status();
                actions.insert(
                    key,
                    ButtonAction {
                        name: routine.title.clone(),
                        title: routine.title.clone(),
                        description: status.label().into(),
                    },
                );
                tiles.push((key, routine_image(&routine.title, status, &inset)));
            }
        }
    }
    if page > 0 {
        actions.insert(
            14,
            ButtonAction {
                name: "Back".into(),
                title: "Previous Routines".into(),
                description: "Previous page of routines".into(),
            },
        );
        tiles.push((14, label_image("BACK", &inset)));
    }
    if pages > 1 {
        actions.insert(
            15,
            ButtonAction {
                name: "More".into(),
                title: "Next Routines".into(),
                description: "Next page of routines".into(),
            },
        );
        let word = if page + 1 < pages { "MORE" } else { "PAGE" };
        tiles.push((
            15,
            label_image(&format!("{word} {}/{pages}", page + 1), &inset),
        ));
    }

    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::MoadimList(total, page);
    }
    for (key, img) in tiles {
        send_button_image(handle, key, &DynamicImage::ImageRgb8(img));
    }
}
