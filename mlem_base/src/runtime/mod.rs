pub mod utils;

use std::{ sync::atomic::Ordering };

use crate::{ PluginImplementationParams, console::ConsoleSender};
use nih_plug::{ prelude::* };
use utils::{ RMS, Timer };

pub struct Runtime {
    pub console: Option<ConsoleSender>,

    sample_rate: f32,
    buffer_size: usize,
    channels: usize,
    clip: bool,
    run_time: RMS
}

impl Runtime {
    pub fn new(console: Option<ConsoleSender>) -> Runtime {
        let runtime = Self {
            console: console,

            sample_rate: 0.0,
            buffer_size: 0,
            channels: 0,
            clip: crate::consts::BUILD_IS_DEBUG,
            run_time: RMS::new(1.0)
        };

        return runtime;
    }
    
    pub fn init(&mut self, sample_rate: f32) {        
        self.sample_rate = sample_rate;
        let execute_timer = Timer::new();

        // User code here

        let execute_time = execute_timer.elapsed_ms();
        self.log(format!("Init in {:.2}ms.", execute_time));
    }

    pub fn reset(&mut self) {
        let execute_timer = Timer::new();

        // User code here

        self.log(format!("Reset in {:.2}ms.", execute_timer.elapsed_ms()));
    }

    pub fn run(&mut self, buffer: &mut Buffer, params: &PluginImplementationParams, transport: &Transport) {
        self.buffer_size = buffer.samples();
        self.channels = buffer.channels();
        let execute_timer = Timer::new();

        // User code here

        if self.clip {
            for channel_samples in buffer.iter_samples() {                        
                for sample in channel_samples {
                    *sample = utils::clip(*sample);
                }
            }
        }

        self.run_time.process( execute_timer.elapsed_ms(), self.sample_rate);
        self.update_params(params);
    }

    pub fn update_params(&mut self, params: &PluginImplementationParams) {
        params.sample_rate.store(self.sample_rate, Ordering::Relaxed);
        params.buffer_size.store(self.buffer_size, Ordering::Relaxed);
        params.channels.store(self.channels, Ordering::Relaxed);
        params.run_ms.store(self.run_time.get(), Ordering::Relaxed);
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