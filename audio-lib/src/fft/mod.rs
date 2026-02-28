mod log_frequency_bins;
mod processor;
pub mod signal_analyzer;

pub fn frequency_step<F: crate::utils::Float>(fft_length: usize, sample_rate: F) -> F {
    assert!(fft_length > 0);
    sample_rate / F::from(fft_length).unwrap()
}

pub use log_frequency_bins::LogFrequencyBin;
pub use log_frequency_bins::LogFrequencyBins;
pub use processor::ProcessingResult;
pub use processor::Processor;
pub use signal_analyzer::{Coefficients, SignalAnalyzer};
