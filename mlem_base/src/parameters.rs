use std::sync::{atomic::AtomicUsize};

use atomic_float::AtomicF32;
use nih_plug::params::Params;

pub trait PluginParameters: Params {
    fn sample_rate(&self) -> &AtomicF32;
    fn buffer_size(&self) -> &AtomicUsize;
    fn channels(&self) -> &AtomicUsize;
    fn run_ms(&self) -> &AtomicF32;

    fn draw(&self);
}