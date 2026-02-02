use nih_plug::prelude::{FloatRange, Param};
use nih_plug_egui::egui::{self, Align, DragValue, Grid, Layout, Response, RichText, Ui, Vec2, WidgetText, widgets};
use crate::interface::param_drag_value::ParamDragValue;

pub const HOVER_HASH: &str = "HOVER";
pub const TOOLTIP_HOVER_WIDTH: f32 = 256.0;
pub const GRID_SPACING: f32 = 4.0;

pub fn help_label(ui: &mut Ui, text: impl Into<RichText>) {    
    ui.add_enabled_ui(false, |ui| {
        ui.label("\u{E3E8}").on_disabled_hover_ui(|ui| {
            ui.set_max_width(TOOLTIP_HOVER_WIDTH);
            ui.monospace(text);
        });
    });
}

pub fn toggle_value(ui: &mut Ui, value: &mut bool, true_text: impl Into<WidgetText>, false_text: impl Into<WidgetText>, size: impl Into<Vec2>) {
    if *value {
        if ui.add_sized(size, egui::SelectableLabel::new(*value, true_text)).clicked() {
            *value = !*value;
        }
    } else {
        if ui.add_sized(size, egui::SelectableLabel::new(*value, false_text)).clicked() {
            *value = !*value;
        }
    }
}

pub fn parameter_grid(ui: &mut Ui, hash: impl std::hash::Hash, add_contents: impl FnOnce(&mut Ui)) {
    Grid::new(hash)
        .num_columns(2)
        .spacing([GRID_SPACING, GRID_SPACING])
        .show(ui, add_contents);
}

pub fn parameter_label(ui: &mut Ui, text: impl Into<WidgetText>, tooltip_text: impl Into<RichText>, add_contents: impl FnOnce(&mut Ui)) {
    ui.label(text).on_hover_ui(|ui| {
        ui.set_max_width(TOOLTIP_HOVER_WIDTH);
        ui.monospace(tooltip_text);
    });

    ui.horizontal(add_contents);
    ui.end_row();
}

pub fn param_info<P: Param>(ui: &mut Ui, param: &P) {
    ui.set_max_width(TOOLTIP_HOVER_WIDTH);
    if param.unit().is_empty() { 
        ui.label(format!("{name}", name = param.name()));
    } else { 
        ui.label(format!("{name} ({unit})", name = param.name(), unit = param.unit()));
    };

    parameter_grid(ui, HOVER_HASH, |ui| {
        ui.label("Value");
        ui.monospace(param.normalized_value_to_string(param.modulated_normalized_value(), true));
        ui.end_row();

        ui.label("Min");
        ui.monospace(param.normalized_value_to_string(0.0, true));
        ui.end_row();

        ui.label("Max");
        ui.monospace(param.normalized_value_to_string(1.0, true));
        ui.end_row();

        ui.label("Default");
        ui.monospace(param.normalized_value_to_string(param.default_normalized_value(), true));
        ui.end_row();
    });
}

pub fn fill_seperator(ui: &mut Ui) {
    let available_size = ui.available_size();
    
    if available_size.x <= 0.0 {
        return;
    }

    ui.add_sized(available_size, widgets::Separator::default().horizontal());
}