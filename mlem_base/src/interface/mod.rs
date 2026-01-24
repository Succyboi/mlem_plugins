pub mod interface_utils;
pub mod param_toggle;

use std::{ hash::Hash, sync::{ Arc, RwLock, atomic::Ordering } };
use mlem_egui_themes::Theme;
use nih_plug::{ plugin, prelude::*, util::gain_to_db };
use nih_plug_egui::{ EguiState, egui::{ self, Align, Context, Layout, Ui } };
use crate::{ PluginImplementation, console::ConsoleReceiver, consts, interface::interface_utils::{help_label, parameter_grid, parameter_label}, metadata::PluginMetadata, parameters::PluginParameters };

const DEFAULT_SPACE: f32 = 4.0;
const LABEL_WIDTH: f32 = 64.0;
const TOP_ID: &str = "Top";
const CONSOLE_MAIN_ID: &str = "Central/Console/Main";
const CONSOLE_ICON: &str = "\u{E47E}";

#[derive(PartialEq)]
pub enum InterfaceCenterViewState {
    About,
    Console,
    Plugin
}

pub struct Interface<T: PluginImplementation<U>, U: PluginParameters> {
    pub console: ConsoleReceiver,
    metadata: PluginMetadata,
    implementation: Arc<T>,
    params: Arc<U>,

    center_view: InterfaceCenterViewState,

    theme: usize,
    themes: [mlem_egui_themes::Theme; 4],
}

impl<T: PluginImplementation<U>, U: PluginParameters> Interface<T, U> {
    pub fn new(metadata: PluginMetadata, implementation: Arc<T>) -> Interface<T, U> {
        let params = implementation.params().clone();

        return Self {
            console: ConsoleReceiver::new(),
            metadata, 
            implementation,
            params: params,

            center_view: InterfaceCenterViewState::Plugin,

            theme: 0,
            themes: [
                mlem_egui_themes::garden_day(),
                mlem_egui_themes::garden_night(),
                mlem_egui_themes::garden_gameboy(),
                mlem_egui_themes::garden_playdate()
            ]
        };
    }

    pub fn create_interface(self, editor_state: Arc<EguiState>) -> Option<Box<dyn Editor>> {
        let interface_lock = Arc::from(RwLock::from(self));
        let interface_lock_build = interface_lock.clone();
        let interface_lock_update = interface_lock.clone();

        return nih_plug_egui::create_egui_editor(
            editor_state,
            (),
            move |egui_ctx, state| {
                let interface = interface_lock_build.clone();

                interface.write().unwrap().build_interface(egui_ctx, state);
            },
            move |egui_ctx, setter, state| {
                let interface = interface_lock_update.clone();

                interface.write().unwrap().draw_interface(egui_ctx, setter, state);
            },
        );
    }

    fn build_interface(&mut self, egui_ctx: &Context, _state: &mut ()) {
        mlem_egui_themes::set_theme(egui_ctx, self.get_theme());

        self.console.log(format!("Initializing {name} v{version}.", name = consts::NAME, version = consts::VERSION));
        self.console.log(format!(""));
        self.console.log(format!("---"));
        self.console.log(format!("{name} \"{description}\" v{version} {build_type} ({id}).", name = self.metadata.name, description = self.metadata.description, version = self.metadata.version, build_type = self.metadata.build_type, id = self.metadata.build_id));
        self.console.log(format!("By {}", self.metadata.authors));
        self.console.log(format!("---"));
        self.console.log(format!(""));

        self.implementation.interface_build();
    }
    
    fn draw_interface(&mut self, egui_ctx: &Context, setter: &ParamSetter, _state: &mut ()) {    
        egui::TopBottomPanel::top(TOP_ID).show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                self.draw_about_button(ui);
                //self.draw_darkmode_toggle(egui_ctx, ui); Not now, implement saving this n stuff

                self.draw_plugin_bar(ui, setter);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    self.draw_console_toggle(ui);
                    ui.separator();
                });
            });
        });

        egui::CentralPanel::default().show(egui_ctx, |ui| {
            self.draw_center(ui, setter);
        });
    }
    
    fn draw_darkmode_toggle(&mut self, egui_ctx: &Context, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            if ui.button("\u{E472}").clicked() {
                self.theme = (self.theme + 1) % self.themes.len();
                mlem_egui_themes::set_theme(egui_ctx, self.get_theme());
            }
        });
    }

    fn draw_console_toggle(&mut self, ui: &mut Ui) {
        let console_updated = self.console.update();

        ui.horizontal(|ui| {
            let button_response = if self.center_view == InterfaceCenterViewState::Console {
                ui.button(format!("{icon} Hide", icon = CONSOLE_ICON))
            } else {
                ui.button(CONSOLE_ICON)
            };
            
            if button_response.clicked() {
                self.center_view = if self.center_view == InterfaceCenterViewState::Console {
                    InterfaceCenterViewState::Plugin
                } else {
                    InterfaceCenterViewState::Console
                };
            }

            if console_updated {
                button_response.highlight();
            }
        });
    }

    fn draw_about_button(&mut self, ui: &mut Ui) {
        let button_response = if self.center_view == InterfaceCenterViewState::About {
            ui.button(format!("{icon} Hide", icon = self.metadata.icon))
        } else {
            ui.button(format!("{icon}", icon = self.metadata.icon))
        };

        if button_response.clicked() {
            self.center_view = if self.center_view == InterfaceCenterViewState::About {
                InterfaceCenterViewState::Plugin
            } else {
                InterfaceCenterViewState::About
            };
        }
    }

    fn draw_center(&mut self, ui: &mut Ui, setter: &ParamSetter) {
        match self.center_view {
            InterfaceCenterViewState::About => self.draw_about(ui),
            InterfaceCenterViewState::Console => self.draw_console(ui, setter, CONSOLE_MAIN_ID),
            InterfaceCenterViewState::Plugin => self.draw_plugin_center(ui, setter),
        }
    }

    fn draw_plugin_center(&mut self, ui: &mut Ui, setter: &ParamSetter) {
        self.implementation.interface_update_center(ui, setter);
    }

    fn draw_plugin_bar(&mut self, ui: &mut Ui, setter: &ParamSetter) {
        self.implementation.interface_update_bar(ui, setter);
    }

    fn draw_console(&mut self, ui: &mut Ui, setter: &ParamSetter, hash: impl Hash) {     
        let params = self.implementation.params();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let load = (params.run_ms().load(Ordering::Relaxed) / (params.buffer_size().load(Ordering::Relaxed) as f32 / params.sample_rate().load(Ordering::Relaxed) * 1000.0) * 100.0).floor();
                let status = format!("({ms:.2}ms / {load:>3}%) @ {rate}hz, {buff}buf, {channels}ch.", 
                    ms = params.run_ms().load(Ordering::Relaxed),
                    load = load, 
                    rate = params.sample_rate().load(Ordering::Relaxed),
                    buff = params.buffer_size().load(Ordering::Relaxed),
                    channels = params.channels().load(Ordering::Relaxed));
        
                    ui.monospace(format!("{}", status));
            });

            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT).with_cross_justify(true), |ui| {
                let log_string = self.console.get_log_string();
                
                egui::ScrollArea::vertical()
                    .id_salt(hash)
                    .show(ui, |ui| {
                        ui.monospace(format!("{}", log_string));
                });
            });
        });
    }

    fn draw_about(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.draw_name(ui);
            ui.label(self.metadata.description);
            ui.separator();
            
            self.draw_info(ui);
        
            ui.separator();
            ui.label("Credits");

            ui.monospace(format!("By {authors}", authors = self.metadata.authors));
            ui.separator();
            ui.monospace(format!("{}", self.metadata.credits));

            ui.separator();
            ui.label("License");
            ui.monospace(format!("{}", self.metadata.license_contents));        
        });
    }

    fn draw_name(&mut self, ui: &mut Ui) {
        ui.heading(format!("{icon} {name}", icon = self.metadata.icon, name = self.metadata.name));
    }

    fn draw_info(&mut self, ui: &mut Ui) {
        ui.label(format!("v{version} {profile} ({id})", version = self.metadata.version, profile = self.metadata.build_type, id = self.metadata.build_id));
        ui.horizontal(|ui| {
            ui.label("By");
            ui.hyperlink_to(self.metadata.vendor, self.metadata.homepage_url);
        });
    }

    fn get_theme(&self) -> Theme {
        return self.themes[self.theme];
    }
}