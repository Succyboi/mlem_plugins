#[derive(Clone)]
pub struct RuntimeData {
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub channels: usize,
    pub run_ms: f32,

    pub active_time_ms: f32,
    pub lufs_global_loudness: f64,
    pub lufs_momentary_loudness: f64,
    pub lufs_range_loudness: f64,
    pub lufs_shortterm_loudness: f64,
}

impl RuntimeData {
    pub fn new() -> RuntimeData {
        Self {
            sample_rate: 0.0,
            buffer_size: 0,
            channels: 0,
            run_ms: 0.0,
            
            active_time_ms: 0.0,
            lufs_global_loudness: 0.0,
            lufs_momentary_loudness: 0.0,
            lufs_range_loudness: 0.0,
            lufs_shortterm_loudness: 0.0,
        }
    }
}