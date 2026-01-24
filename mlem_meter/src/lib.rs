pub mod consts;
pub mod runtime;

use atomic_float::{ AtomicF32, AtomicF64 };
use mlem_base::{interface::interface_utils::{parameter_grid, parameter_label}, metadata::PluginMetadata, parameters::PluginParameters};
use runtime::{ Runtime };
use mlem_base::{ interface::{ Interface }, PluginImplementation };
use nih_plug::prelude::*;
use std::sync::{ Arc, atomic::{AtomicBool, AtomicUsize, Ordering} };
use nih_plug_egui::{EguiState, egui::{Align, Layout, Ui}};
use consts::PLUGIN_METADATA;

pub struct Meter {
    runtime: Runtime,
    params: Arc<MeterParams>
}

#[derive(Params)]
pub struct MeterParams {
    #[persist = "editor-state"] editor_state: Arc<EguiState>,
    #[id = "reset_on_play"]     reset_on_play: BoolParam,
    
    sample_rate: AtomicF32,
    buffer_size: AtomicUsize,
    channels: AtomicUsize,
    run_ms: AtomicF32,
    
    reset_meter: AtomicBool,
    active_time_ms: AtomicF32,
    lufs_global_loudness: AtomicF64,
    lufs_momentary_loudness: AtomicF64,
    lufs_range_loudness: AtomicF64,
    lufs_shortterm_loudness: AtomicF64
}

impl Default for Meter {
    fn default() -> Self {
        let runtime = Runtime::new(None);

        Self {
            runtime: runtime,
            params: Arc::new(MeterParams::default())
        }
    }
}

impl Default for MeterParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(PLUGIN_METADATA.window_width, PLUGIN_METADATA.window_height),
            reset_on_play: BoolParam::new("Reset On Play", true),

            reset_meter: AtomicBool::new(false),
            sample_rate: AtomicF32::new(0.0),
            buffer_size: AtomicUsize::new(0),
            channels: AtomicUsize::new(0),
            run_ms: AtomicF32::new(0.0),

            active_time_ms: AtomicF32::new(0.0),
            lufs_global_loudness: AtomicF64::new(0.0),
            lufs_momentary_loudness: AtomicF64::new(0.0),
            lufs_range_loudness: AtomicF64::new(0.0),
            lufs_shortterm_loudness: AtomicF64::new(0.0)
        }
    }
}

impl PluginParameters for MeterParams {
    fn sample_rate(&self) -> &AtomicF32 { &self.sample_rate }
    fn buffer_size(&self) -> &AtomicUsize { &self.buffer_size }
    fn channels(&self) -> &AtomicUsize { &self.channels }
    fn run_ms(&self) -> &AtomicF32 { &self.run_ms }
}

impl Meter {
    // TODO move to trait implementation.

    fn interface_center(&self) -> impl Fn(&Meter, &mut Ui, &ParamSetter)  + 'static + Send + Sync {
        return move |meter: &Meter, ui, setter: &ParamSetter| {
            parameter_grid(ui, "Meters", |ui| {
                parameter_label(ui, "Integrated", "Loudness total since reset.", |ui| {
                    ui.monospace(format!("{: >6.2} lufs", meter.params.lufs_global_loudness.load(Ordering::Relaxed)));
                });

                parameter_label(ui, "Momentary", "Loudness over a duration of 0.4 seconds.", |ui| {
                    ui.monospace(format!("{: >6.2} lufs", meter.params.lufs_momentary_loudness.load(Ordering::Relaxed)));
                });

                parameter_label(ui, "Short Term", "Loudness over a duration of 3 seconds.", |ui| {
                    ui.monospace(format!("{: >6.2} lufs", meter.params.lufs_shortterm_loudness.load(Ordering::Relaxed)));
                });

                parameter_label(ui, "Range", "Loudness range total since reset.", |ui| {
                    ui.monospace(format!("{: >6.2} lufs", meter.params.lufs_range_loudness.load(Ordering::Relaxed)));
                });
            });
            
            ui.add_space(ui.available_height() - 12.0);
            ui.horizontal(|ui| {
                let seconds = meter.params.active_time_ms.load(Ordering::Relaxed) / 1000.0;
                let minutes = f32::floor(seconds / 60.0);
                
                if ui.button("Reset").clicked() {
                    meter.params.reset_meter.store(true, Ordering::Relaxed);
                }
                ui.monospace(format!("{minutes: >1.0}m{seconds: >1.0}s", minutes = minutes, seconds = seconds - minutes * 60.0));
                
                ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                    let mut reset_on_play_value = meter.params.reset_on_play.value();
            
                    if ui.checkbox(&mut reset_on_play_value, "Reset On Play").clicked() {
                        setter.begin_set_parameter(&meter.params.reset_on_play);
                        setter.set_parameter(&meter.params.reset_on_play, reset_on_play_value);
                        setter.end_set_parameter(&meter.params.reset_on_play);
                    }
                });
            });
        };
    }
}

impl PluginImplementation for Meter {
    fn metadata(&self) -> PluginMetadata {
        return PLUGIN_METADATA;
    }

    fn params(&self) -> Arc<dyn PluginParameters> {
        return self.params.clone();
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
        let editor_state = self.params.editor_state.clone();
        let params = self.params.clone();
        let center_draw =  self.interface_center();
        let interface = Interface::new(consts::PLUGIN_METADATA, params, center_draw);
        
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
