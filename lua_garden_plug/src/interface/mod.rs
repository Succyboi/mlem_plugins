pub mod interface_data;
pub mod interface_utils;
pub mod interface_module;
pub mod interface_runtime;
pub mod parameter;

use std::{ hash::Hash, sync::{ Arc, RwLock } };
use interface_runtime::{InterfaceRuntime, InterfaceRuntimeView};
use mlem_egui_themes::Theme;
use nih_plug::prelude::*;
use nih_plug_egui::{ egui::{ self, Context, Ui }, EguiState };
use interface_data::InterfaceData;
use crate::{ consts, ConsoleReceiver, runtime::{library, workspace::Workspace}, LuaGardenParams, runtime::runtime_data::RuntimeState, RuntimeData };

const DEFAULT_SPACE: f32 = 4.0;
const BAR_HEIGHT: f32 = 20.0;
const TOP_ID: &str = "Top";
const BOTTOM_ID: &str = "Bottom";
const DEFAULT_MENU_WIDTH: f32 = 128.0;
const ABOUT_MENU_WIDTH: f32 = 320.0;
const ABOUT_LICENSE_SCROLL_HEIGHT: f32 = 320.0;
const CONSOLE_MAIN_ID: &str = "Central/Console/Main";
const CONSOLE_HEIGHT: f32 = 128.0;
const CONSOLE_PREVIEW_CHARS: usize = 40;
const DRAFT_EDITOR_ID: &str = "Central/DraftEditor";
const LOAD_BUTTON_WIDTH: f32 = 64.0;

pub struct Interface {
    pub console: ConsoleReceiver,

    show_create_workspace: bool,
    show_open_workspace: bool,
    center_view: CenterView,
    show_console: bool,
    draft_code_selection: RuntimeCode,

    create_workspace_path: String,
    open_workspace_path: String,

    interface_runtime: InterfaceRuntime,

    theme: usize,
    themes: [mlem_egui_themes::Theme; 4],
}

#[derive(PartialEq)]
pub enum CenterView {
    Code,
    Interface
}

#[derive(PartialEq, Clone)]
pub enum InterfaceMode {
    Draft,
    Workspace
}

#[derive(PartialEq)]
pub enum RuntimeCode {
    Init,
    Reset,
    Run,
    Trigger,
    Interface
}

impl Interface {
    pub fn new() -> Interface {
        return Self {
            console: ConsoleReceiver::new(),

            show_create_workspace: false,
            show_open_workspace: false,
            center_view: CenterView::Code,
            show_console: true,
            draft_code_selection: RuntimeCode::Run,

            create_workspace_path: library::default_workspaces_path(),
            open_workspace_path: library::default_workspaces_path(),

            interface_runtime: InterfaceRuntime::new(),

            theme: 0,
            themes: [
                mlem_egui_themes::garden_night(),
                mlem_egui_themes::garden_day(),
                mlem_egui_themes::garden_gameboy(),
                mlem_egui_themes::garden_playdate()
            ]
        };
    }

    pub fn create_interface(self, editor_state: Arc<EguiState>, params: Arc<LuaGardenParams>, runtime_data_lock: Arc<RwLock<RuntimeData>>, interface_data_lock: Arc<RwLock<InterfaceData>>) -> Option<Box<dyn Editor>> {
        let interface_lock = Arc::from(RwLock::from(self));
        let interface_lock_build = interface_lock.clone();
        let interface_lock_update = interface_lock.clone();
        let runtime_data_lock_build = runtime_data_lock.clone();
        let runtime_data_lock_update = runtime_data_lock.clone();
        let interface_data_lock_build = interface_data_lock.clone();
        let interface_data_lock_update = interface_data_lock.clone();
        let params_build = params.clone();
        let params_update = params.clone();

        return nih_plug_egui::create_egui_editor(
            editor_state,
            (),
            move |egui_ctx, _state| {
                let params_build = params_build.clone();
                let interface = interface_lock_build.clone();
                let runtime_data = runtime_data_lock_build.clone();
                let interface_data = interface_data_lock_build.clone();

                interface.write().unwrap().build_interface(egui_ctx, _state, params_build, runtime_data, interface_data);
            },
            move |egui_ctx, _setter, _state| {
                let params_update = params_update.clone();
                let interface = interface_lock_update.clone();
                let runtime_data = runtime_data_lock_update.clone();
                let interface_data = interface_data_lock_update.clone();

                interface.write().unwrap().draw_interface(egui_ctx, _setter, _state, params_update, runtime_data, interface_data);
            },
        );
    }

    fn build_interface(&mut self, egui_ctx: &Context, _state: &mut (), _params: Arc<LuaGardenParams>, _runtime_data: Arc<RwLock<RuntimeData>>, _interface_data: Arc<RwLock<InterfaceData>>) {
        mlem_egui_themes::set_theme(egui_ctx, self.get_theme());

        self.console.log(format!("{name} v{version} {build_type} ({id}).", name = consts::NAME, version = consts::VERSION, build_type = consts::BUILD_TYPE, id = consts::BUILD_ID));
        self.console.log(format!("{}", consts::MOTD));
    }
    
    fn draw_interface(&mut self, egui_ctx: &Context, _setter: &ParamSetter, _state: &mut (), _params: Arc<LuaGardenParams>, runtime_data: Arc<RwLock<RuntimeData>>, interface_data: Arc<RwLock<InterfaceData>>) {    
        let runtime_data = runtime_data.read().unwrap().clone();
        let mut interface_data = interface_data.write().unwrap();
        
        interface_data.update_from_runtime(&runtime_data);

        egui::TopBottomPanel::top(TOP_ID).show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                self.draw_about_button(ui);
                self.draw_mode_menu(ui, &mut interface_data);
                ui.separator();
                self.draw_center_view_selection(ui, &mut interface_data);
                ui.separator();
                self.draw_modules_menu(ui, &mut interface_data);
    
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    self.draw_panic_button(ui, &runtime_data, &mut interface_data);
                });
            });
        });

        egui::CentralPanel::default().show(egui_ctx, |ui| {
            if interface_data.runtime_target_state == RuntimeState::Offline {
                self.center_view = CenterView::Code;
            }
            
            match self.center_view {
                CenterView::Code => {
                    match interface_data.mode {
                        InterfaceMode::Draft => {
                            self.draw_draft_editor(ui, &runtime_data, &mut interface_data);
                        },
                        InterfaceMode::Workspace => {
                            self.draw_workspace_editor(ui, &runtime_data, &mut interface_data);
                        }
                    }
                },
                CenterView::Interface => {
                    self.draw_module_interface(ui, &runtime_data, &mut interface_data);
                }
            }

            self.draw_console(ui, &runtime_data, CONSOLE_MAIN_ID);
        });

        egui::TopBottomPanel::bottom(BOTTOM_ID).show(egui_ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                self.draw_darkmode_toggle(egui_ctx, ui);
                self.draw_console_toggle(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    self.draw_signal_flow(ui, &runtime_data, &mut interface_data);
                });
            });
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
        if self.show_console { return; }

        let log = self.console.get_last_log();
        let log_string = if log.len() > CONSOLE_PREVIEW_CHARS {
            format!("{}...", &log[..CONSOLE_PREVIEW_CHARS].trim())
        }else {
            String::from(log[..].trim())
        };

        ui.horizontal(|ui| {
            if ui.button("Console").clicked() {
                self.show_console = !self.show_console;
            }

            ui.monospace(log_string);
        });
    }

    fn draw_about_button(&mut self, ui: &mut Ui) {
        ui.menu_button(format!("v{}", consts::VERSION), |ui| {
            ui.set_max_width(DEFAULT_MENU_WIDTH);    

            ui.menu_button("About", |ui|{
                ui.set_max_width(ABOUT_MENU_WIDTH);
                self.draw_info(ui);
                ui.separator();
                self.draw_disclaimer(ui);
            });

            ui.menu_button("License", |ui| {
                ui.set_max_width(ABOUT_MENU_WIDTH);    

                egui::ScrollArea::vertical().max_height(ABOUT_LICENSE_SCROLL_HEIGHT).show(ui, |ui| {
                    ui.monospace(format!("{}", consts::LICENSE_CONTENTS));        
                });
            });

            ui.menu_button("Credits", |ui| {
                ui.set_max_width(ABOUT_MENU_WIDTH);    

                egui::ScrollArea::vertical().max_height(ABOUT_LICENSE_SCROLL_HEIGHT).show(ui, |ui| {
                    ui.monospace(format!("By {authors}\n{description}", authors = consts::AUTHORS, description = consts::DESCRIPTION));
                    ui.separator();
                    ui.monospace(format!("{}", consts::CREDITS));        
                });
            });
        });
    }
    
    fn draw_center_view_selection(&mut self, ui: &mut Ui, interface_data: &mut InterfaceData) {
        ui.selectable_value(&mut self.center_view, CenterView::Code, "Code");
        
        ui.add_enabled_ui(interface_data.runtime_target_state != RuntimeState::Offline, |ui| {
            ui.selectable_value(&mut self.center_view, CenterView::Interface, "Interface")
            .on_disabled_hover_ui(|ui| {
                ui.set_max_width(interface_utils::TOOLTIP_HOVER_WIDTH);
                ui.monospace("Load a module to show interface.");
            });
        });
    }
    
    fn draw_mode_menu(&mut self, ui: &mut Ui, interface_data: &mut InterfaceData) {
        ui.menu_button("Mode", |ui| {
            ui.set_max_width(DEFAULT_MENU_WIDTH);

            if ui.selectable_value(&mut interface_data.mode, InterfaceMode::Draft, "Draft").clicked() {
                ui.close_menu();
            }
            if ui.selectable_value(&mut interface_data.mode, InterfaceMode::Workspace, "Workspace").clicked() {
                ui.close_menu();
            }
        });
    }

    fn draw_modules_menu(&mut self, ui: &mut Ui, interface_data: &mut InterfaceData) {
        if interface_data.mode == InterfaceMode::Workspace { return; }

        ui.menu_button("Modules", |ui| {
            if ui.button("Empty").clicked() {
                interface_data.draft_content = library::MODULE_DEFAULT.to_module_content();
                ui.close_menu();
            }

            ui.menu_button("Examples", |ui| {
                ui.set_max_width(DEFAULT_MENU_WIDTH);
    
                for e in 0..library::MODULE_EXAMPLES.len() {
                    if ui.button(format!("{index} {name}", index = e, name = library::MODULE_EXAMPLES[e].1)).clicked() {
                        interface_data.draft_content = library::MODULE_EXAMPLES[e].0.to_module_content();
                        ui.close_menu();
                    }
                }
            });
        });
    }

    fn draw_panic_button(&mut self, ui: &mut Ui, runtime_data: &RuntimeData, interface_data: &mut InterfaceData) {
        if runtime_data.state == RuntimeState::Online {
            if ui.button("\u{E4E4} PANIC").clicked() {
                interface_data.set_runtime_target_state(RuntimeState::Clear);
            }
        } else {
            ui.set_enabled(false);
            if ui.button("\u{E4E4} PANIC").clicked() {
                interface_data.set_runtime_target_state(RuntimeState::Clear);
            }
            ui.set_enabled(true);
        }
    }

    fn draw_signal_flow(&mut self, ui: &mut Ui, runtime_data: &RuntimeData, interface_data: &mut InterfaceData) {
        let mut clip = interface_data.runtime_clip;
        interface_utils::toggle_value(ui, &mut clip, "\u{EA9E} Clip", "\u{EA9A} No clip", [LOAD_BUTTON_WIDTH, ui.available_height()]);

        if clip != interface_data.runtime_clip {
            interface_data.set_runtime_clip(clip);
        }

        match runtime_data.state {
            RuntimeState::Online => { 
                ui.label("\u{E1D7}");
                ui.label(format!("{}", &runtime_data.module_name)).on_hover_text(format!("{name} by {author}\n\"{description}\"", 
                    name = &runtime_data.module_name,
                    author = &runtime_data.module_author,
                    description = &runtime_data.module_description));
                ui.label("\u{E1D7}");
            }
            _ => {
                ui.label("\u{E1D7}");
                ui.label("Module");            
                ui.label("\u{E1D7}");
            }
        }

        let mut input_noise = interface_data.runtime_input_noise;
        interface_utils::toggle_value(ui, &mut input_noise, "\u{E1B4} Noise", "\u{E802} Input", [LOAD_BUTTON_WIDTH, ui.available_height()]);
        interface_data.set_runtime_input_noise(input_noise);

        ui.separator();
    }
    
    fn draw_draft_editor(&mut self, ui: &mut Ui, runtime_data: &RuntimeData, interface_data: &mut InterfaceData) {
        ui.horizontal(|ui| {
            ui.label("Draft");
            interface_utils::help_label(ui, format!("In draft mode, {name} loads from the code you write in the included code editor.\n\
            Note however, all is lost when you exit.", name = consts::NAME));
            ui.separator();
    
            ui.selectable_value(&mut self.draft_code_selection, RuntimeCode::Init, "Init");
            ui.selectable_value(&mut self.draft_code_selection, RuntimeCode::Reset, "Reset");
            ui.selectable_value(&mut self.draft_code_selection, RuntimeCode::Trigger, "Trigger");
            ui.selectable_value(&mut self.draft_code_selection, RuntimeCode::Run, "Run");
            ui.selectable_value(&mut self.draft_code_selection, RuntimeCode::Interface, "Interface");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                self.draw_load_button(ui, runtime_data, interface_data);
            });
        });
    
        ui.add_space(DEFAULT_SPACE);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let code = match self.draft_code_selection {
                RuntimeCode::Init => (&mut interface_data.draft_content.init, library::INIT_PATH),
                RuntimeCode::Reset => (&mut interface_data.draft_content.reset, library::RESET_PATH),
                RuntimeCode::Trigger => (&mut interface_data.draft_content.trigger, library::TRIGGER_PATH),
                RuntimeCode::Run => (&mut interface_data.draft_content.run, library::RUN_PATH),
                RuntimeCode::Interface => (&mut interface_data.draft_content.interface, library::INTERFACE_PATH)
            };

            let height = if self.show_console {
                ui.available_height() - CONSOLE_HEIGHT - DEFAULT_SPACE
            } else {
                ui.available_height() - BAR_HEIGHT
            };
            ui.add_sized([ui.available_width(), height], egui::TextEdit::multiline(code.0)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .id(Into::into(format!("{prefix}/{id}", prefix = DRAFT_EDITOR_ID, id = code.1))),
            );
        });
    }
    
    fn draw_workspace_editor(&mut self, ui: &mut Ui, runtime_data: &RuntimeData, interface_data: &mut InterfaceData) {
        ui.horizontal(|ui| {
            ui.label("Workspace");
            interface_utils::help_label(ui, format!("In workspace mode, {name} loads from lua files in a folder.\n\
                This way you can use a code editor of your preference.", name = consts::NAME));
            ui.separator();

            ui.horizontal(|ui| {
                if self.show_create_workspace {
                    if ui.button("Cancel").clicked() {
                        self.show_create_workspace(false);
                        self.show_open_workspace(false);
                    }

                    if ui.button("Create").clicked() {
                        let path = self.create_workspace_path.clone();
                        match Workspace::create_at_path(path, None) {
                            Ok(w) => {
                                self.show_create_workspace(false);
                                self.show_open_workspace(false);
                                interface_data.workspace = Some(w);
                            },
                            Err(e) => {
                                self.console.log(format!("Failed to create workspace: {}", e));
                            }
                        }
                    }

                    if ui.button("Create from draft").clicked() {
                        let path = self.create_workspace_path.clone();
                        let content = interface_data.draft_content.clone();
                        match Workspace::create_at_path(path, Some(content)) {
                            Ok(w) => {
                                self.show_create_workspace(false);
                                self.show_open_workspace(false);
                                interface_data.workspace = Some(w);
                            },
                            Err(e) => {
                                self.console.log(format!("Failed to create workspace: {}", e));
                            }
                        }
                    }
                } else if self.show_open_workspace {
                    if ui.button("Cancel").clicked() {
                        self.show_create_workspace(false);
                        self.show_open_workspace(false);
                    }

                    if ui.button("Open").clicked() {
                        let path = self.create_workspace_path.clone();
                        match Workspace::load_from_path(path) {
                            Ok(w) => {
                                self.show_create_workspace(false);
                                self.show_open_workspace(false);
                                interface_data.workspace = Some(w);
                            },
                            Err(e) => {
                                self.console.log(format!("Failed to open workspace: {}", e));
                            }
                        }
                    }
                } else {
                    if ui.button("Create").clicked() {
                        self.show_create_workspace(true);
                        self.show_open_workspace(false);
                    }
    
                    if ui.button("Open").clicked() {
                        self.show_open_workspace(true);
                        self.show_create_workspace(false);
                    }
                }
            });
    
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                self.draw_load_button(ui, runtime_data, interface_data);
            });
        });

        ui.add_space(DEFAULT_SPACE * 4.0);

        if self.show_create_workspace {
            ui.horizontal(|ui| {
                ui.label("Create a workspace at");
                ui.text_edit_singleline(&mut self.create_workspace_path);
                ui.label(".");
            });
        } else if self.show_open_workspace {
            ui.horizontal(|ui| {
                ui.label("Open a workspace at");
                ui.text_edit_singleline(&mut self.open_workspace_path);
                ui.label(".");
            });
        } else {
            match &interface_data.workspace {
                Some(workspace) => {
                    ui.horizontal(|ui| {
                        ui.label("Workspace loaded at");
                        ui.monospace(format!("{}", workspace.path));
                        ui.label(".");
                    });
                },
                None => {
                    ui.label("No workspace loaded. Create or open one.");
                }
            }
        }

        ui.add_space(DEFAULT_SPACE * 4.0);
    }
    
    fn draw_module_interface(&mut self, ui: &mut Ui, runtime_data: &RuntimeData, interface_data: &mut InterfaceData) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.interface_runtime.view, InterfaceRuntimeView::Interface, "Interface");
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.interface_runtime.view, InterfaceRuntimeView::Parameters, "Parameters")
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                self.draw_load_button(ui, runtime_data, interface_data);
            });
        });

        ui.add_space(DEFAULT_SPACE);

        self.interface_runtime.draw(ui, runtime_data, interface_data);
    }
    
    fn draw_load_button(&mut self, ui: &mut Ui, runtime_data: &RuntimeData, interface_data: &mut InterfaceData) {
        let enabled = interface_data.mode == InterfaceMode::Draft || interface_data.workspace != None;

        ui.add_enabled_ui(enabled, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                match runtime_data.state {
                    RuntimeState::Offline => {
                        if ui.add_sized([LOAD_BUTTON_WIDTH, ui.available_height()], egui::Button::new("\u{E52E} Load")).clicked() {
                            self.update_workspace(interface_data);
                            interface_data.set_runtime_target_state(RuntimeState::Refresh);
                        }
                    },
                    RuntimeState::Online => {
                        if ui.add_sized([LOAD_BUTTON_WIDTH, ui.available_height()], egui::Button::new("\u{E522} Reload")).clicked() {
                            self.update_workspace(interface_data);
                            interface_data.set_runtime_target_state(RuntimeState::Refresh);
                        }
                    }
                    _ => {
                        ui.set_enabled(false);
                        let _ = ui.add_sized([LOAD_BUTTON_WIDTH, ui.available_height()], egui::Button::new("..."));
                        ui.set_enabled(true);
                    }
                }
            });
        });
    }
    
    fn draw_console(&mut self, ui: &mut Ui, runtime_data: &RuntimeData, hash: impl Hash) {
        if !self.show_console { return; }
        
        ui.add_space(ui.available_height() - CONSOLE_HEIGHT);

        ui.vertical(|ui| {
            ui.set_max_height(ui.available_height() - BAR_HEIGHT);

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Console").clicked() {
                    self.show_console = !self.show_console;
                }
         
                ui.monospace(":");
        
                let load = (runtime_data.run_ms / (runtime_data.buffer_size as f32 / runtime_data.sample_rate * 1000.0) * 100.0).floor();
                let status = format!("({ms:.2}ms / {load:>3}%) at {rate}hz, {buff} samples, {channels} channels.", 
                    ms = runtime_data.run_ms,
                    load = load, 
                    rate = runtime_data.sample_rate,
                    buff = runtime_data.buffer_size,
                    channels = runtime_data.channels);
        
                match runtime_data.state {
                    RuntimeState::Offline => {
                        ui.monospace("No module loaded.");
                    },
                    RuntimeState::Online => {
                        ui.monospace(format!("Running {}", status));
                    },
                    _ => {
                        ui.monospace("...");
                    }
                }
            });
        
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT).with_cross_justify(true), |ui| {
                let log_string = self.console.get_log_string();
                
                egui::ScrollArea::vertical()
                    .id_source(hash)
                    .show(ui, |ui| {
                        ui.monospace(format!("{}", log_string));
                });
            });
        });
    }

    fn draw_info(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(format!("{icon} {name}", icon = consts::ICON, name = consts::NAME));
            ui.label(format!("v{version} {profile} ({id})", version = consts::VERSION, profile = consts::BUILD_TYPE, id = consts::BUILD_ID));
        });
    }

    fn draw_disclaimer(&mut self, ui: &mut Ui) {
        ui.label(consts::DISCLAIMER);
    }

    fn show_create_workspace(&mut self, show: bool) {
        self.show_create_workspace = show;
    }

    fn show_open_workspace(&mut self, show: bool) {
        self.show_open_workspace = show;
    }

    fn get_theme(&self) -> Theme {
        return self.themes[self.theme];
    }

    fn update_workspace(&mut self, interface_data: &mut InterfaceData) {
        match &mut interface_data.workspace {
            Some(workspace) => {
                match workspace.update() {
                    Ok(()) => (),
                    Err(e) => self.console.log(format!("Couldn't update workspace: {}", e))
                }
            },
            None => ()
        }
    }
}