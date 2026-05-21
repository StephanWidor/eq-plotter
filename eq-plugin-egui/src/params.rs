use crate::*;
use audio_lib::*;
use egui_lib::{colors, options};
use std::ops::RangeInclusive;
use std::sync::{self, atomic};

// hm, can we somehow get rid of this without destroying the nice::Enum and nice::Params derive?
use nice_plug::params::Params;

#[derive(nice::Params)]
pub struct PluginParams<
    const NUM_BANDS: usize,
    const NUM_CHANNELS: usize,
    const ANALYZER_NUM_BINS: usize,
> {
    #[persist = "editor_state"]
    pub editor_state: sync::Arc<nice_plug_egui::EguiState>,

    #[nested(array, group = "eq_params")]
    pub eq_params: [EqParams; NUM_BANDS],

    pub sample_rate: nice::AtomicF32,
    pub show_gain: atomic::AtomicBool,
    pub show_signal_gain_spectrum: atomic::AtomicBool,
    pub show_phase: atomic::AtomicBool,
    pub show_impulse_response: atomic::AtomicBool,
    pub show_poles_and_zeros: atomic::AtomicBool,
    pub selected_eq_index: atomic::AtomicUsize,
    pub analyzer_data:
        fft::signal_analyzer::SharedData<f32, { ANALYZER_NUM_BINS }, { NUM_CHANNELS }>,
    pub eq_ranges: EqRanges,
    pub impulse_response_settings: ImpulseResponseSettings,
    pub color_palette: colors::ColorPalette,
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    pub fn new(settings: &Settings<NUM_BANDS>, smoothing_length_ms: f32) -> Self {
        let eq_ranges = settings.eq_ranges.clone();
        Self {
            editor_state: nice_plug_egui::EguiState::from_size(1000, 700),
            eq_params: std::array::from_fn(|index| {
                EqParams::from_eq(
                    format!(" [{}]", index + 1).as_str(),
                    &settings.init_eqs[index],
                    &eq_ranges.log_frequency_range,
                    &eq_ranges.db_range,
                    &eq_ranges.q_range,
                    smoothing_length_ms,
                )
            }),
            sample_rate: nice::AtomicF32::new(settings.init_sample_rate),
            show_gain: atomic::AtomicBool::new(true),
            show_signal_gain_spectrum: atomic::AtomicBool::new(true),
            show_phase: atomic::AtomicBool::new(false),
            show_impulse_response: atomic::AtomicBool::new(false),
            show_poles_and_zeros: atomic::AtomicBool::new(false),
            selected_eq_index: atomic::AtomicUsize::new(usize::MAX),
            analyzer_data: fft::signal_analyzer::SharedData::new(settings.init_sample_rate),
            eq_ranges: eq_ranges,
            impulse_response_settings: settings.impulse_response.clone(),
            color_palette: colors::ColorPalette::default(),
        }
    }

    pub fn eqs<F: utils::Float>(&self) -> [eq::Eq<F>; NUM_BANDS] {
        std::array::from_fn(|index| self.eq_params[index].to_eq())
    }

    pub fn show_options(&self) -> options::ShowOptions {
        options::ShowOptions {
            gain: self.show_gain.load(atomic::Ordering::Relaxed),
            signal_gain_spectrum: self
                .show_signal_gain_spectrum
                .load(atomic::Ordering::Relaxed),
            phase: self.show_phase.load(atomic::Ordering::Relaxed),
            impulse_response: self.show_impulse_response.load(atomic::Ordering::Relaxed),
            poles_and_zeros: self.show_poles_and_zeros.load(atomic::Ordering::Relaxed),
        }
    }

    pub fn set_show_options(&self, options: &options::ShowOptions) {
        self.show_gain
            .store(options.gain, atomic::Ordering::Relaxed);
        self.show_signal_gain_spectrum
            .store(options.signal_gain_spectrum, atomic::Ordering::Relaxed);
        self.show_phase
            .store(options.phase, atomic::Ordering::Relaxed);
        self.show_impulse_response
            .store(options.impulse_response, atomic::Ordering::Relaxed);
        self.show_poles_and_zeros
            .store(options.poles_and_zeros, atomic::Ordering::Relaxed);
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct EqTypeWrapper {
    eq_type: eq::EqType,
}

impl From<eq::EqType> for EqTypeWrapper {
    fn from(eq_type: eq::EqType) -> Self {
        Self { eq_type: eq_type }
    }
}

impl Into<eq::EqType> for EqTypeWrapper {
    fn into(self) -> eq::EqType {
        self.eq_type
    }
}

impl nice::Enum for EqTypeWrapper {
    fn variants() -> &'static [&'static str] {
        &eq::EqType::ALL_NAMES
    }

    fn ids() -> Option<&'static [&'static str]> {
        None
    }

    fn to_index(self) -> usize {
        self.eq_type as usize
    }

    fn from_index(index: usize) -> Self {
        let from_result = eq::EqType::try_from(index);
        match from_result {
            Ok(eq_type) => Self { eq_type: eq_type },
            _ => Self {
                eq_type: eq::EqType::try_from(0).unwrap(),
            },
        }
    }
}

pub type EqTypeParam = nice::EnumParam<EqTypeWrapper>;

#[derive(nice::Params)]
pub struct EqParams {
    #[id = "gain_db"]
    pub gain_db: nice::FloatParam,

    #[id = "frequency"]
    pub log_frequency: nice::FloatParam,

    #[id = "q"]
    pub q: nice::FloatParam,

    #[id = "eq_type"]
    pub eq_type: EqTypeParam,
}

impl EqParams {
    fn from_eq(
        names_suffix: &str,
        eq: &eq::Eq<f32>,
        log_frequency_range: &RangeInclusive<f32>,
        db_range: &RangeInclusive<f32>,
        q_range: &RangeInclusive<f32>,
        smoothing_length_ms: f32,
    ) -> Self {
        Self {
            gain_db: nice::FloatParam::new(
                format!("Gain (dB){names_suffix}"),
                eq.gain.db(),
                nice::FloatRange::Linear {
                    min: *db_range.start(),
                    max: *db_range.end(),
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(smoothing_length_ms))
            .with_unit(" dB"),
            log_frequency: nice::FloatParam::new(
                format!("Frequency (Hz){names_suffix}"),
                eq.frequency.log_hz(),
                nice::FloatRange::Linear {
                    min: *log_frequency_range.start(),
                    max: *log_frequency_range.end(),
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(smoothing_length_ms))
            .with_unit(" Hz")
            .with_value_to_string(sync::Arc::new(egui_lib::utils::log_frequency_to_string))
            .with_string_to_value(sync::Arc::new(egui_lib::utils::string_to_log_frequency)),
            q: nice::FloatParam::new(
                format!("q{names_suffix}"),
                eq.q,
                nice::FloatRange::Linear {
                    min: *q_range.start(),
                    max: *q_range.end(),
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(smoothing_length_ms)),
            eq_type: EqTypeParam::new(
                format!("Eq Type{names_suffix}"),
                EqTypeWrapper::from(eq.eq_type),
            ),
        }
    }

    pub fn to_eq<F: audio_lib::utils::Float>(&self) -> eq::Eq<F> {
        eq::Eq {
            gain: eq::Gain::Db(F::from(self.gain_db.value()).unwrap()),
            frequency: eq::Frequency::LogHz(F::from(self.log_frequency.value()).unwrap()),
            q: F::from(self.q.value()).unwrap(),
            eq_type: self.eq_type.value().into(),
        }
    }

    pub fn set_gain_db<F: utils::Float>(&self, gain_db: F, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.gain_db);
        setter.set_parameter(&self.gain_db, gain_db.to_f32().unwrap());
        setter.end_set_parameter(&self.gain_db);
    }

    pub fn set_log_frequency<F: utils::Float>(
        &self,
        log_frequency: F,
        setter: &nice::ParamSetter<'_>,
    ) {
        setter.begin_set_parameter(&self.log_frequency);
        setter.set_parameter(&self.log_frequency, log_frequency.to_f32().unwrap());
        setter.end_set_parameter(&self.log_frequency);
    }

    pub fn set_q<F: utils::Float>(&self, q: F, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.q);
        setter.set_parameter(&self.q, q.to_f32().unwrap());
        setter.end_set_parameter(&self.q);
    }

    pub fn set_eq_type(&self, eq_type: eq::EqType, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.eq_type);
        setter.set_parameter(&self.eq_type, eq_type.into());
        setter.end_set_parameter(&self.eq_type);
    }
}
