use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use image::DynamicImage;
use rusb::{Context, DeviceHandle};

use crate::domain::actions::{ButtonAction, CentyProject, ScreenView};
use crate::infrastructure::images::{
    generate_project_item_image, generate_search_image, generate_sort_image,
};
use crate::infrastructure::usb::send_button_image;
use crate::presentation::tui;

pub fn render_project_list(
    projects: &[CentyProject],
    page: usize,
    filter: Option<&str>,
    handle: &DeviceHandle<Context>,
    state: &Arc<Mutex<tui::AppState>>,
) {
    const PER_PAGE: usize = 10;
    let start = page * PER_PAGE;
    let page_slice: Vec<&CentyProject> = projects.iter().skip(start).take(PER_PAGE).collect();

    let mut actions: HashMap<u8, ButtonAction> = HashMap::new();
    for (i, project) in page_slice.iter().enumerate() {
        let key = (i + 1) as u8;
        actions.insert(
            key,
            ButtonAction {
                name: project.name.clone(),
                title: project.org.clone(),
                description: project.url.clone(),
            },
        );
    }
    actions.insert(
        14,
        ButtonAction {
            name: "Sort".into(),
            title: "Sort Projects".into(),
            description: "Sort projects by name".into(),
        },
    );
    actions.insert(
        15,
        ButtonAction {
            name: "Search".into(),
            title: "Search".into(),
            description: "Open Spotlight search".into(),
        },
    );

    let count = page_slice.len();
    {
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        s.actions = actions;
        s.screen = ScreenView::CentyProjectList {
            total: projects.len(),
            page,
            filter: filter.map(|f| f.to_string()),
        };
    }

    let project_img = DynamicImage::ImageRgb8(generate_project_item_image());
    for i in 0..count {
        send_button_image(handle, (i + 1) as u8, project_img.clone());
    }
    send_button_image(handle, 14, DynamicImage::ImageRgb8(generate_sort_image()));
    send_button_image(handle, 15, DynamicImage::ImageRgb8(generate_search_image()));
}
