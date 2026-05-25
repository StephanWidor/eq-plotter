use audio_lib::*;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Settings<F: audio_lib::utils::Float> {
    pub eq_ranges: EqRanges<F>,
    pub impulse_response_params: ImpulseResponseParams<F>,
    pub show_options: ShowOptions,
}

#[derive(Debug, Clone)]
pub struct ShowOptions {
    pub gain: bool,
    pub signal_gain_spectrum: bool,
    pub phase: bool,
    pub impulse_response: bool,
    pub poles_and_zeros: bool,
}

#[derive(Debug, Clone)]
pub struct EqRanges<F: utils::Float> {
    pub db_range: RangeInclusive<F>,
    pub log_frequency_range: RangeInclusive<F>,
    pub q_range: RangeInclusive<F>,
}

impl<F: utils::Float> EqRanges<F> {
    pub fn frequency_range(&self) -> RangeInclusive<F> {
        utils::log_to_frequency(*self.log_frequency_range.start())
            ..=utils::log_to_frequency(*self.log_frequency_range.end())
    }
}

impl<F: utils::Float> Default for EqRanges<F> {
    fn default() -> Self {
        let db_range = F::from(-40).unwrap()..=F::from(40).unwrap();
        let frequency_range = F::from(10).unwrap()..=F::from(20000).unwrap();
        let log_frequency_range = utils::frequency_to_log(*frequency_range.start())
            ..=utils::frequency_to_log(*frequency_range.end());
        Self {
            db_range: db_range,
            log_frequency_range: log_frequency_range,
            q_range: F::from(0.1).unwrap()..=F::from(10).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImpulseResponseParams<F: utils::Float> {
    pub eps: F,
    pub hold_length: usize,
    pub max_length: usize,
}

impl<F: utils::Float> Default for ImpulseResponseParams<F> {
    fn default() -> Self {
        Self {
            eps: F::from(0.001).unwrap(),
            hold_length: 10,
            max_length: 1024,
        }
    }
}

impl ShowOptions {
    pub fn new_all_enabled() -> Self {
        Self {
            gain: true,
            signal_gain_spectrum: true,
            phase: true,
            impulse_response: true,
            poles_and_zeros: true,
        }
    }

    pub fn new_only_gain() -> Self {
        Self {
            gain: true,
            signal_gain_spectrum: true,
            phase: false,
            impulse_response: false,
            poles_and_zeros: false,
        }
    }
}
