use eframe::egui;

use vibe_keyboard::domain::profile::Widget;

/// List of side widgets: reorder, remove, edit text, and add one of each kind.
/// Widgets replace the side label and image when the list is not empty.
pub(super) fn widget_editor(ui: &mut egui::Ui, widgets: &mut Vec<Widget>) {
    ui.separator();
    ui.strong("Widgets");
    let last = widgets.len().saturating_sub(1);
    let (mut remove, mut swap) = (None, None);
    for (i, widget) in widgets.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(widget_name(widget));
            if let Widget::Text { value } = widget {
                ui.text_edit_singleline(value);
            }
            if ui.add_enabled(i > 0, egui::Button::new("Up")).clicked() {
                swap = Some((i, i - 1));
            }
            if ui
                .add_enabled(i < last, egui::Button::new("Down"))
                .clicked()
            {
                swap = Some((i, i + 1));
            }
            if ui.button("Remove").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        widgets.remove(i);
    }
    if let Some((a, b)) = swap {
        widgets.swap(a, b);
    }
    ui.horizontal_wrapped(|ui| {
        for (label, widget) in [
            ("+ CPU %", Widget::CpuPercent),
            ("+ CPU bar", Widget::CpuBar),
            ("+ Mem bar", Widget::MemBar),
            ("+ CPU|Mem bar", Widget::CpuMemBar),
            ("+ Temp", Widget::Temperature),
            ("+ Disk", Widget::DiskFree),
            (
                "+ Text",
                Widget::Text {
                    value: String::new(),
                },
            ),
        ] {
            if ui.button(label).clicked() {
                widgets.push(widget);
            }
        }
    });
}

pub(super) const fn widget_name(widget: &Widget) -> &'static str {
    match widget {
        Widget::CpuPercent => "CPU %",
        Widget::CpuBar => "CPU bar",
        Widget::MemBar => "Mem bar",
        Widget::CpuMemBar => "CPU|Mem bar",
        Widget::Temperature => "Temperature",
        Widget::DiskFree => "Disk free",
        Widget::Text { .. } => "Text",
    }
}
