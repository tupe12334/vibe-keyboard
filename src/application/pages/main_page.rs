use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::infrastructure::usb::Device;
use image::DynamicImage;
use tracing::info;

use crate::domain::actions::{ButtonAction, ScreenView};
use crate::infrastructure::images::page_images;
use crate::infrastructure::persistence::DeviceState;
use crate::infrastructure::profile_store;
use crate::infrastructure::usb::send_button_image;
use crate::presentation::tui;

/// The hard-coded pages from before profiles. Kept only as the reference for `default_from_legacy`.
#[cfg(test)]
pub fn page_actions(page: usize) -> HashMap<u8, ButtonAction> {
    let mut map = HashMap::new();
    match page {
        0 => {
            map.insert(
                1,
                ButtonAction {
                    name: "Centy".into(),
                    title: "List Projects".into(),
                    description: "Show centy projects".into(),
                },
            );
            map.insert(
                2,
                ButtonAction {
                    name: "Terminal".into(),
                    title: "Open Terminal".into(),
                    description: "Launch a new terminal window".into(),
                },
            );
            map.insert(
                3,
                ButtonAction {
                    name: "Claude".into(),
                    title: "Open Claude".into(),
                    description: "Launch terminal with claude --allow-dangerously-skip-permissions"
                        .into(),
                },
            );
            map.insert(
                4,
                ButtonAction {
                    name: "Centy Web".into(),
                    title: "Open Centy".into(),
                    description: "Open app.centy.io in Chrome".into(),
                },
            );
        }
        1 => {
            map.insert(
                1,
                ButtonAction {
                    name: "NumPad".into(),
                    title: "Input Number".into(),
                    description: "Open numpad entry screen".into(),
                },
            );
            map.insert(
                14,
                ButtonAction {
                    name: "Log File".into(),
                    title: "Open Log".into(),
                    description: "Open app.log in VS Code".into(),
                },
            );
            map.insert(
                15,
                ButtonAction {
                    name: "VSCode Config".into(),
                    title: "Open Config".into(),
                    description: "Open config file in VS Code".into(),
                },
            );
        }
        _ => {}
    }
    map
}

pub fn render_main_page(
    page: usize,
    handle: &Device,
    state: &Arc<Mutex<tui::AppState>>,
    dev_state: &Arc<Mutex<DeviceState>>,
) {
    let keys = {
        let s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.profile
            .pages
            .get(page)
            .map_or_else(Default::default, |p| p.keys.clone())
    };
    let actions: HashMap<u8, ButtonAction> = keys
        .iter()
        .map(|(key, cfg)| {
            (
                *key,
                ButtonAction {
                    name: cfg.label.clone(),
                    title: cfg.action.kind().label().into(),
                    description: String::new(),
                },
            )
        })
        .collect();
    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.actions = actions;
        s.screen = ScreenView::MainPage { page };
    }
    {
        let mut ds = dev_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        ds.current_page = page;
        ds.save();
    }
    // The same images the editor previews. Keys 11–13 are the navigation art.
    let root = profile_store::root_dir();
    for (key, img) in page_images(&keys, &root) {
        send_button_image(handle, key, DynamicImage::ImageRgb8(img));
    }
    info!("page {}: {} key(s)", page + 1, keys.len());
}

#[cfg(test)]
mod tests {
    use super::page_actions;
    use crate::domain::profile::{Action, Profile};

    #[test]
    fn default_from_legacy_matches_page_actions() {
        let profile = Profile::default_from_legacy();
        let expected_name = |name: &str| match name {
            "Centy" => "centy_projects",
            "Terminal" => "terminal",
            "Claude" => "claude",
            "Centy Web" => "centy_web",
            "NumPad" => "numpad",
            "Log File" => "log_file",
            "VSCode Config" => "vscode_config",
            other => panic!("unmapped legacy name {other}"),
        };
        for page in 0..2 {
            let legacy = page_actions(page);
            assert_eq!(
                profile.pages[page].keys.len(),
                legacy.len(),
                "page {page} key count"
            );
            for (key, button) in legacy {
                let cfg = profile.key_at(page, key);
                assert_eq!(cfg.map(|k| k.label.as_str()), Some(button.title.as_str()));
                assert_eq!(
                    cfg.map(|k| k.action.clone()),
                    Some(Action::Builtin {
                        name: expected_name(&button.name).into()
                    })
                );
            }
        }
    }
}
