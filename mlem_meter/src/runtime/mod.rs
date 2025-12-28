pub mod utils;
pub mod runtime_data;

use core::fmt;
use std::{ fmt::Error, sync::atomic::Ordering };

use crate::{ PluginImplementationParams, console::ConsoleSender, runtime::runtime_data::RuntimeData};
use nih_plug::{ prelude::* };
use utils::{ RMS, Timer };
use ebur128::{ EbuR128, Mode };

pub struct Runtime {
    pub console: Option<ConsoleSender>,

    sample_rate: f32,
    buffer_size: usize,
    channels: usize,
    meter_id: usize,
    last_playing: bool,

    active_time: Timer,
    lufs_global_loudness: f64,
    lufs_momentary_loudness: f64,
    lufs_range_loudness: f64,
    lufs_shortterm_loudness: f64,

    run_time: RMS,
    ebur128: Option<EbuR128>,
    clip: bool
}

impl Runtime {
    pub fn new(console: Option<ConsoleSender>) -> Runtime {
        let runtime = Self {
            console: console,

            sample_rate: 0.0,
            buffer_size: 0,
            channels: 0,
            meter_id: 0,
            last_playing: false,

            active_time: Timer::new(),
            lufs_global_loudness: 0.0,
            lufs_momentary_loudness: 0.0,
            lufs_range_loudness: 0.0,
            lufs_shortterm_loudness: 0.0,
            
            run_time: RMS::new(1.0),
            ebur128: None,
            clip: crate::consts::BUILD_IS_DEBUG
        };

        return runtime;
    }
    
    pub fn init(&mut self, sample_rate: f32) {        
        self.sample_rate = sample_rate;

        let execute_timer = Timer::new();
        let execute_time = execute_timer.elapsed_ms();
        
        self.log(format!("Init in {:.2}ms.", execute_time));
    }

    pub fn reset(&mut self) {
        let execute_timer = Timer::new();

        match self.reset_meter() {
            Ok(()) => (),
            Err(e) => self.log(format!("Failed to reset meter: {}", e))
        }

        self.log(format!("Reset in {:.2}ms.", execute_timer.elapsed_ms()));
    }

    pub fn run(&mut self, buffer: &mut Buffer, params: &PluginImplementationParams, transport: &Transport) {
        self.buffer_size = buffer.samples();
        self.channels = buffer.channels();
        let execute_timer = Timer::new();

        if params.reset_on_play.value() && !self.last_playing && transport.playing {        
            match self.reset_meter() {
                Ok(()) => (),
                Err(e) => {
                    self.log(format!("Failed to reset meter: {}", e));
                }
            }
        }
        self.last_playing = transport.playing;

        if params.reset_meter.load(Ordering::SeqCst) {
            match self.reset_meter() {
                Ok(()) => (),
                Err(e) => self.log(format!("Couldn't refresh EbuR128: {}", e))
            }

            params.reset_meter.store(false, Ordering::SeqCst);
        }

        match self.run_ebur128(buffer) {
            Ok(()) => (),
            Err(e) => {
                self.log(format!("Failed to run EbuR128: {}", e));
            }
        }

        if self.clip {
            for channel_samples in buffer.iter_samples() {                        
                for sample in channel_samples {
                    *sample = utils::clip(*sample);
                }
            }
        }

        self.run_time.process( execute_timer.elapsed_ms(), self.sample_rate);
    }

    pub fn update_runtime_data(&mut self, runtime_data: &mut RuntimeData) {
        runtime_data.sample_rate = self.sample_rate;
        runtime_data.buffer_size = self.buffer_size;
        runtime_data.channels = self.channels;
        runtime_data.run_ms = self.run_time.get();

        runtime_data.active_time_ms = self.active_time.elapsed_ms();

        runtime_data.lufs_global_loudness = self.lufs_global_loudness;
        runtime_data.lufs_momentary_loudness = self.lufs_momentary_loudness;
        runtime_data.lufs_range_loudness = self.lufs_range_loudness;
        runtime_data.lufs_shortterm_loudness = self.lufs_shortterm_loudness;
    }

    fn run_ebur128(&mut self, buffer: &mut Buffer) -> Result<(), ebur128::Error> {
        match &mut self.ebur128 {
            Some(_ebur128) => (),
            None => {
                self.reset_meter()?;
            }
        };

        for block_channel in buffer.iter_blocks(buffer.samples()) {     
            for channel in 0..block_channel.1.channels() {
                match block_channel.1.get(channel) {
                    Some(samples) => {
                        let ebur128 = self.ebur128.as_mut().expect("No EbuR128.");
                        ebur128.add_frames_f32(samples)?;
                    },
                    None => {
                        self.log(format!("Could not get samples from block."));
                    }
                };
            }
        }

        let ebur128 = self.ebur128.as_ref().expect("No EbuR128.");
        self.lufs_global_loudness = ebur128.loudness_global()?;
        self.lufs_momentary_loudness = ebur128.loudness_momentary()?;
        self.lufs_range_loudness = ebur128.loudness_range()?;
        self.lufs_shortterm_loudness = ebur128.loudness_shortterm()?;

        Ok(())
    }

    fn reset_meter(&mut self) -> Result<(), ebur128::Error>  {
        self.ebur128 = Some(EbuR128::new(self.channels as u32, self.sample_rate as u32, Mode::all())?);
        self.active_time.reset();

        Ok(())
    }

    fn log(&self, message: String) {
        match &self.console {
            Some(c) => {
                c.log(message);
            },
            None => {
                println!("No console exists for Runtime. Log not registered by receiver: {}", message)
            }
        }
    }
}