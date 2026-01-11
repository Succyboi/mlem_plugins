use std::sync::{atomic::AtomicUsize};

use atomic_float::AtomicF32;

pub trait Parameters {
    fn sample_rate(&self) -> &AtomicF32;
    fn buffer_size(&self) -> &AtomicUsize;
    fn channels(&self) -> &AtomicUsize;
    fn run_ms(&self) -> &AtomicF32;
}