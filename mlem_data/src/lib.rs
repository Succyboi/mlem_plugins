pub mod consts;
pub mod runtime;
pub mod read_mode;

use atomic_float::{ AtomicF32, AtomicF64 };
use egui_file::FileDialog;
use mlem_base::{console::ConsoleSender, interface::{self, param_combo_box, param_drag_value::ParamDragValue, param_toggle, utils::{parameter_grid, parameter_label, toggle_value}}, metadata::PluginMetadata, parameters::PluginParameters};
use runtime::{ Runtime };
use mlem_base::{ interface::{ Interface }, PluginImplementation };
use nih_plug::prelude::*;
use std::{ffi::OsStr, ops::Deref, path::{Path, PathBuf}, str::FromStr, sync::{ Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering} }};
use nih_plug_egui::{EguiState, egui::{Align, Context, Layout, Ui, Vec2}};
use consts::PLUGIN_METADATA;

use crate::read_mode::DataReadMode;

pub const DATA_PREVIEW_SIZE: usize = 61 * 7;

pub struct Meter {
    runtime: Runtime,
    params: Arc<MeterParams>,
    implementation: Arc<MeterImplementation>
}

#[derive(Params)]
pub struct MeterParams {
    #[persist = "editor-state"] editor_state: Arc<EguiState>,
    #[id = "mute"]              mute: BoolParam,
    #[id = "mono"]              mono: BoolParam,
    #[id = "read-mode"]         read_mode: EnumParam<DataReadMode>,
    
    sample_rate: AtomicF32,
    buffer_size: AtomicUsize,
    channels: AtomicUsize,
    run_ms: AtomicF32,
    
    load_path: Mutex<Option<String>>,
    data_preview: Mutex<[u8; DATA_PREVIEW_SIZE]>
}

pub struct MeterImplementation { 
    params: Arc<MeterParams>,
    open_file_dialog: Mutex<FileDialog>
}

impl Default for Meter {
    fn default() -> Self {
        let runtime = Runtime::new(None);
        let params = Arc::new(MeterParams::default());

        Self {
            runtime: runtime,
            params: params.clone(),
            implementation: Arc::new(MeterImplementation::new(params.clone()))
        }
    }
}

impl Default for MeterParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(PLUGIN_METADATA.window_width, PLUGIN_METADATA.window_height),

            mute: BoolParam::new("Mute", true),
            mono: BoolParam::new("Mono", false),
            read_mode: EnumParam::new("Read Mode", DataReadMode::Bit8),

            sample_rate: AtomicF32::new(0.0),
            buffer_size: AtomicUsize::new(0),
            channels: AtomicUsize::new(0),
            run_ms: AtomicF32::new(0.0),

            load_path: Mutex::from(None),
            data_preview: Mutex::from([0; DATA_PREVIEW_SIZE])
        }
    }
}

impl PluginParameters for MeterParams {
    fn sample_rate(&self) -> &AtomicF32 { &self.sample_rate }
    fn buffer_size(&self) -> &AtomicUsize { &self.buffer_size }
    fn channels(&self) -> &AtomicUsize { &self.channels }
    fn run_ms(&self) -> &AtomicF32 { &self.run_ms }
}

impl Meter { }

impl PluginImplementation<MeterParams> for MeterImplementation {
    fn new(params: Arc<MeterParams>) -> MeterImplementation {
        return Self {
            params: params.clone(),
            open_file_dialog: Mutex::from(FileDialog::open_file(None)
                .resizable(false)
                .default_size(Vec2::new(PLUGIN_METADATA.window_width  as f32 - interface::DEFAULT_SPACE * 8.0, PLUGIN_METADATA.window_height as f32 / 2.0))
                .show_rename(false))
        }
    }

    fn metadata(&self) -> PluginMetadata {
        return PLUGIN_METADATA;
    }

    fn params(&self) -> Arc<MeterParams> {
        return self.params.clone();
    }

    fn interface_build(&self, _ctx: &Context) { }

    fn interface_update_center(&self, ui: &mut Ui, _ctx: &Context, setter: &ParamSetter) {
        ui.horizontal(|ui| {
            ui.add(param_toggle::ParamToggle::for_param(&self.params.mono, setter, "Mono", "Stereo"));
            ui.add(param_combo_box::ParamComboBox::for_param(&self.params.read_mode, setter));
        });
        
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.monospace(consts::DISCLAIMER);
            ui.separator();

            let data_preview = *self.params.data_preview.lock().unwrap();
            let mut data_string = String::new();
            for i in 0..data_preview.len() {
                data_string.push_str(format!("{:02X?}", data_preview[i]).as_str());
            }
            ui.monospace(data_string);
            ui.separator();
        });
    }

    fn interface_update_bar(&self, ui: &mut Ui, ctx: &Context, setter: &ParamSetter) {
        self.bar_mute(ui, setter);
        self.bar_open(ui, ctx);
    }
}

impl MeterImplementation {
    fn bar_mute(&self, ui: &mut Ui, setter: &ParamSetter) {
        ui.add(param_toggle::ParamToggle::for_param(&self.params.mute, setter, "Mute", "Mute"));
    }

    fn bar_open(&self, ui: &mut Ui, ctx: &Context) {
        let mut open_file_dialog = self.open_file_dialog.lock().unwrap();
        if (ui.button("Open")).clicked() {
            open_file_dialog.open();
        }

        if open_file_dialog.show(ctx).selected() {
            if let Some(path) = open_file_dialog.path() {
                let Ok(path) = String::from_str(&path.to_string_lossy());
                let mut load_path = self.params.load_path.lock().unwrap();
                *load_path = Some(path);
            }
        }
    }
}

impl Plugin for Meter {
    const NAME: &'static str = PLUGIN_METADATA.name;
    const VENDOR: &'static str = PLUGIN_METADATA.vendor;
    const URL: &'static str = PLUGIN_METADATA.homepage_url;
    const EMAIL: &'static str = PLUGIN_METADATA.email;
    const VERSION: &'static str = PLUGIN_METADATA.version;

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let interface = Interface::new(consts::PLUGIN_METADATA, self.implementation.clone());
        
        let editor_state = self.params.editor_state.clone();
        self.runtime.console = Some(interface.console.create_sender());
        let editor = interface.create_interface(editor_state);

        return editor;
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let _ = self.runtime.init(buffer_config.sample_rate);

        return true;
    }

    fn reset(&mut self) {
        self.runtime.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let params = self.params.clone();

        self.runtime.run(buffer, &params, context.transport());

        return ProcessStatus::Normal;
    }
}

impl ClapPlugin for Meter {
    const CLAP_ID: &'static str = PLUGIN_METADATA.identifier;
    const CLAP_DESCRIPTION: Option<&'static str> = Some(PLUGIN_METADATA.description);
    const CLAP_MANUAL_URL: Option<&'static str> = Some(PLUGIN_METADATA.homepage_url);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(PLUGIN_METADATA.support_url);

    const CLAP_FEATURES: &'static [ClapFeature] = PLUGIN_METADATA.clap_features;
}

impl Vst3Plugin for Meter {
    const VST3_CLASS_ID: [u8; 16] = PLUGIN_METADATA.class_identifier;

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = PLUGIN_METADATA.vst3_subcategories;
}

nih_export_clap!(Meter);
nih_export_vst3!(Meter);
