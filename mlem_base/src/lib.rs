pub mod consts;
pub mod runtime;
pub mod interface;
pub mod console;

use atomic_float::{ AtomicF32, AtomicF64 };
use console::ConsoleReceiver;
use runtime::{ Runtime };
use interface::{ Interface };
use nih_plug::prelude::*;
use std::sync::{ Arc, atomic::{AtomicBool, AtomicUsize} };
use nih_plug_egui::EguiState;

pub struct PluginImplementation {
    runtime: Runtime,
    params: Arc<PluginImplementationParams>
}

#[derive(Params)]
pub struct PluginImplementationParams {
    #[persist = "editor-state"] editor_state: Arc<EguiState>,
    #[id = "reset_on_play"]     reset_on_play: BoolParam,
    
    reset_meter: AtomicBool,
    sample_rate: AtomicF32,
    buffer_size: AtomicUsize,
    channels: AtomicUsize,
    run_ms: AtomicF32,

    active_time_ms: AtomicF32,
    lufs_global_loudness: AtomicF64,
    lufs_momentary_loudness: AtomicF64,
    lufs_range_loudness: AtomicF64,
    lufs_shortterm_loudness: AtomicF64
}

impl Default for PluginImplementation {
    fn default() -> Self {
        let runtime = Runtime::new(None);

        Self {
            runtime: runtime,
            params: Arc::new(PluginImplementationParams::default())
        }
    }
}

impl Default for PluginImplementationParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(consts::WINDOW_SIZE_WIDTH, consts::WINDOW_SIZE_HEIGHT),
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

impl PluginImplementation { }

impl Plugin for PluginImplementation {
    const NAME: &'static str = consts::NAME;
    const VENDOR: &'static str = consts::PLUGIN_VENDOR;
    const URL: &'static str = consts::HOMEPAGE;
    const EMAIL: &'static str = consts::PLUGIN_EMAIL;
    const VERSION: &'static str = consts::VERSION;

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
        let interface = Interface::new();
        
        self.runtime.console = Some(interface.console.create_sender());
        let editor = interface.create_interface(editor_state, params);

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

impl ClapPlugin for PluginImplementation {
    const CLAP_ID: &'static str = consts::PLUGIN_ID;
    const CLAP_DESCRIPTION: Option<&'static str> = Some(consts::DESCRIPTION);
    const CLAP_MANUAL_URL: Option<&'static str> = Some(consts::HOMEPAGE);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(consts::DESCRIPTION);

    const CLAP_FEATURES: &'static [ClapFeature] = consts::PLUGIN_CLAP_FEATURES;
}

impl Vst3Plugin for PluginImplementation {
    const VST3_CLASS_ID: [u8; 16] = consts::PLUGIN_CLASS_ID;

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = consts::PLUGIN_VST3_SUBCATEGORIES;
}

nih_export_clap!(PluginImplementation);
nih_export_vst3!(PluginImplementation);
