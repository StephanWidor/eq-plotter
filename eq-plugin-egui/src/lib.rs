pub mod editor;
pub mod params;
pub mod plugin;
pub mod processor;

pub mod config {
    pub const NUM_BANDS: usize = 8;
    pub const MAX_NUM_CHANNELS: usize = 2;
    pub const ANALYZER_NUM_BINS: usize = 12;
    pub const DEFAULT_ANALYZER_COEFFICIENTS: audio_lib::fft::signal_analyzer::Coefficients<f32> =
        audio_lib::fft::signal_analyzer::Coefficients {
            sample_rate: 48000.0,
            attack_time: 0.01,
            release_time: 0.1,
            window_type: audio_lib::windows::WindowType::VonHann,
        };
    pub const SMOOTHING_LENGTH_MS: f32 = 20.0;
}
