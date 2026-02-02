use core::fmt;
use std::fmt::format;
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::fs::FileExt;
use std::{ fmt::Error, sync::atomic::Ordering };
use mlem_base::console::ConsoleSender;
use nih_plug_egui::egui::load;
use crate::consts;
use crate::{ MeterParams };
use nih_plug::{ prelude::* };
use mlem_base::runtime::utils::{ self, RMS, Timer };

const MAX_DATA_SIZE: usize = 1 * 1024 * 1024; // 1 Megabyte

// TODO Figure out how to synthesize from bits
pub struct Runtime {
    pub console: Option<ConsoleSender>,

    sample_rate: f32,
    buffer_size: usize,
    channels: usize,
    last_playing: bool,

    file_path: Option<String>,
    file_offset: u64,
    data: [u8; MAX_DATA_SIZE],
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
            
            file_path: None,
            file_offset: 0,
            data: [0; MAX_DATA_SIZE],
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

        let _ = self.update_data_from_array(consts::DEFAULT_DATA);
        
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
            self.file_path = Some(path.clone());
            if let Err(err) = self.update_data_from_file() {
                self.log(format!("Failed to load file at path \"{path}\": {err}"));
            }
            self.file_offset = 0;
        }
        *load_path = None;

        let mut data_preview = params.data_preview.lock().unwrap();
        let len = usize::min(self.data_len - self.data_pos, data_preview.len());
        data_preview[0..len].clone_from_slice(&self.data[self.data_pos..(len + self.data_pos)]);

        for channel_samples in buffer.iter_samples() {          
            let mut value = if params.mono.value() { self.next_data() } else { 0.0 };

            for sample in channel_samples {
                if !params.mono.value() {
                    value = self.next_data();
                }

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

    fn next_data(&mut self) -> f32 {
        let raw = self.data[self.data_pos];
        let value = raw as f32 / u8::MAX as f32 * 2.0 - 0.5;
        self.data_pos = self.data_pos + 1;

        if self.data_pos >= self.data_len {
            let _ = self.update_data_from_file();
            self.data_pos = 0;
        }
        
        return value;
    }

    fn update_data_from_file(&mut self) -> std::io::Result<()> {
        if let Some(file_path) = &self.file_path {
            let file = File::open(file_path)?;
            let file_len = file.metadata()?.len();

            if file_len == 0 {
                return Err(std::io::Error::new(io::ErrorKind::Other, "File length is 0."));
            }

            self.file_offset = (self.file_offset + self.data_len as u64) % file_len;
            self.data_len = file.read_at(&mut self.data, self.file_offset)?;
            self.data_pos = 0;

            self.log(format!("File read {bytes} bytes at {percent}%", bytes = self.data_len, percent = f32::floor(self.file_offset as f32 / file.metadata()?.len() as f32 * 100.0)));
        }
        
        Ok(())
    }

    fn update_data_from_array(&mut self, array: &[u8]) -> std::io::Result<()> {
        let len = usize::min(array.len(), MAX_DATA_SIZE);

        self.data[0..len].clone_from_slice(&array[0..len]);
        self.data_len = len;
        self.data_pos = 0;

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