use crate::*;
use audio_lib::*;
use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Settings<F: utils::Float> {
    pub eq_ranges: EqRanges<F>,
    pub impulse_response_params: ImpulseResponseParams<F>,
    pub init_show_options: ShowOptions,
    pub color_palette: ui::colors::ColorPalette,
}

#[derive(Debug, Clone)]
pub struct EqRanges<F: utils::Float> {
    pub db_range: RangeInclusive<F>,
    pub log_frequency_range: RangeInclusive<F>,
    pub q_range: RangeInclusive<F>,
}

#[derive(Debug, Clone)]
pub struct ImpulseResponseParams<F: utils::Float> {
    pub rel_eps: F,
    pub release_time: F,
    pub max_time: F,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShowOptions {
    pub gain: bool,
    pub signal_gain_spectrum: bool,
    pub phase: bool,
    pub impulse_response: bool,
    pub poles_and_zeros: bool,
}

impl<F: utils::Float> Default for Settings<F> {
    fn default() -> Self {
        Self {
            eq_ranges: EqRanges::default(),
            impulse_response_params: ImpulseResponseParams::default(),
            init_show_options: ShowOptions::new_all_enabled(),
            color_palette: ui::colors::ColorPalette::default(),
        }
    }
}

impl<F: utils::Float> EqRanges<F> {
    pub fn frequency_range(&self) -> RangeInclusive<F> {
        utils::log_to_frequency(*self.log_frequency_range.start())
            ..=utils::log_to_frequency(*self.log_frequency_range.end())
    }
}

impl<F: utils::Float> Default for EqRanges<F> {
    fn default() -> Self {
        let db_range = F::from_integral(-40)..=F::from_integral(40);
        let frequency_range = F::from_integral(10)..=F::from_integral(20000);
        let log_frequency_range = utils::frequency_to_log(*frequency_range.start())
            ..=utils::frequency_to_log(*frequency_range.end());
        Self {
            db_range: db_range,
            log_frequency_range: log_frequency_range,
            q_range: F::from_float(0.1)..=F::from_integral(10),
        }
    }
}

impl<F: utils::Float> Default for ImpulseResponseParams<F> {
    fn default() -> Self {
        Self {
            rel_eps: F::from_float(0.01),
            release_time: F::from_float(0.01),
            max_time: F::from_float(0.5),
        }
    }
}

impl ShowOptions {
    pub const fn new_all_enabled() -> Self {
        Self {
            gain: true,
            signal_gain_spectrum: true,
            phase: true,
            impulse_response: true,
            poles_and_zeros: true,
        }
    }

    pub const fn new_only_gain() -> Self {
        Self {
            gain: true,
            signal_gain_spectrum: true,
            phase: false,
            impulse_response: false,
            poles_and_zeros: false,
        }
    }
}
