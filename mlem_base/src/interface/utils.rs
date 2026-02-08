use std::{fs::File, io::BufWriter, path::Path, sync::Arc};

use nih_plug::prelude::{FloatRange, Param};
use nih_plug_egui::egui::{self, Align, ColorImage, Context, DragValue, Grid, Layout, Response, RichText, Ui, UserData, Vec2, WidgetText, epaint::image, widgets};
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

pub fn save_screenshot(ui: &Ui, ctx: &Context) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(UserData::default()));

    ui.input(|i| {
        for event in &i.raw.events {
            if let egui::Event::Screenshot { image, .. } = event {
                let path_raw = format!("{:?}/scr.png", std::env::home_dir());
                let path = Path::new(&path_raw);
                save_image(&image, path);
            }
        }
    });
}

pub fn save_image(image: &ColorImage, path: &Path) {
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image.width() as u32, image.height() as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header().unwrap();

    let mut data = Vec::new();
    for pixel in image.pixels.clone() {
        data.push(pixel.r());
        data.push(pixel.g());
        data.push(pixel.b());
        data.push(pixel.a());
    }

    writer.write_image_data(&data).unwrap();

    // TODO log that screenshot was saved. I need a better logger...
}