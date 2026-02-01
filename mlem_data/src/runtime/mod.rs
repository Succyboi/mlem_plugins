pub mod utils;

use core::fmt;
use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::{ fmt::Error, sync::atomic::Ordering };
use mlem_base::console::ConsoleSender;
use crate::consts;
use crate::{ MeterParams };
use nih_plug::{ prelude::* };
use utils::{ RMS, Timer };

const MAX_BUFFER_SIZE: usize = 1 * 1024 * 1024; // 1 Megabyte

pub struct Runtime {
    pub console: Option<ConsoleSender>,

    sample_rate: f32,
    buffer_size: usize,
    channels: usize,
    last_playing: bool,

    file: Option<File>,
    data: [u8; MAX_BUFFER_SIZE],
    data_len: usize,
    data_pos: usize,

    run_time: RMS,
}

impl Runtime {
    pub fn new(console: Option<ConsoleSender>) -> Runtime {
        let runtime = Self {
            console: console,

            sample_rate: 0.0,
            buffer_size: 0,
            channels: 0,
            last_playing: false,
            
            file: None,
            data: [0; MAX_BUFFER_SIZE],
            data_len: 0,
            data_pos: 0,

            run_time: RMS::new(1.0),
        };

        return runtime;
    }
    
    pub fn init(&mut self, sample_rate: f32) {        
        self.sample_rate = sample_rate;

        let execute_timer = Timer::new();
        let execute_time = execute_timer.elapsed_ms();

        self.update_buffer_from_array(consts::DEFAULT_DATA);
        
        self.log(format!("Init in {:.2}ms.", execute_time));
    }

    pub fn reset(&mut self) {
        let execute_timer = Timer::new();

        self.log(format!("Reset in {:.2}ms.", execute_timer.elapsed_ms()));
    }

    pub fn run(&mut self, buffer: &mut Buffer, params: &MeterParams, transport: &Transport) {
        self.buffer_size = buffer.samples();
        self.channels = buffer.channels();
        let execute_timer = Timer::new();

        self.last_playing = transport.playing;


        let mut load_path = params.load_path.lock().unwrap();
        if let Some(path) = (*load_path).clone() {
            self.file = match Self::load_file(&path) {
                Ok(file) => {
                    Some(file)
                },
                Err(_) => None
            }
        }
        *load_path = None;

        for channel_samples in buffer.iter_samples() {                        
            for sample in channel_samples {
                let value = self.data[self.data_pos] as f32 / u8::MAX as f32 * 2.0 - 0.5;
                self.data_pos = (self.data_pos + 1) % self.data_len;

                // TODO keep loading from file

                if params.mute.value() {
                    *sample = 0.0;
                    continue;
                }

                *sample = utils::clip(value);
            }
        }

        self.run_time.process( execute_timer.elapsed_ms(), self.sample_rate);
        self.update_params(params);
    }

    pub fn update_params(&mut self, params: &MeterParams) {
        params.sample_rate.store(self.sample_rate, Ordering::Relaxed);
        params.buffer_size.store(self.buffer_size, Ordering::Relaxed);
        params.channels.store(self.channels, Ordering::Relaxed);
        params.run_ms.store(self.run_time.get(), Ordering::Relaxed);
    }

    fn load_file(path: &String) -> std::io::Result<File> {
        let file = File::open(path)?;

        Ok(file)
    }

    fn update_buffer_from_file(mut self) -> std::io::Result<()> {
        match self.file {
            None => (),
            Some(mut file) => {
                self.buffer_size = file.read(&mut self.data)?;
            }
        }

        Ok(())
    }

    fn update_buffer_from_array(&mut self, array: &[u8]) -> std::io::Result<()> {
        let len = usize::min(consts::DEFAULT_DATA.len(), MAX_BUFFER_SIZE);

        self.data[0..len].clone_from_slice(&consts::DEFAULT_DATA[0..len]);
        self.data_len = len;

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