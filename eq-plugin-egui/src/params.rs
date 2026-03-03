use crate::*;
use audio_lib::*;
use eq_plotter_egui::{colors, options};
use nih_plug::prelude as nih;
use std::ops::RangeInclusive;
use std::sync::{self, atomic};

// hm, can we somehow get rid of this without destroying the nih::Enum and nih::Params derive?
use nih_plug::params::Params;

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

impl nih::Enum for EqTypeWrapper {
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

pub type EqTypeParam = nih::EnumParam<EqTypeWrapper>;

#[derive(nih::Params)]
pub struct EqParams {
    #[id = "gain_db"]
    pub gain_db: nih::FloatParam,

    #[id = "frequency"]
    pub log_frequency: nih::FloatParam,

    #[id = "q"]
    pub q: nih::FloatParam,

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
    ) -> Self {
        Self {
            gain_db: nih::FloatParam::new(
                format!("Gain (dB){names_suffix}"),
                eq.gain.db(),
                nih::FloatRange::Linear {
                    min: *db_range.start(),
                    max: *db_range.end(),
                },
            )
            .with_smoother(nih::SmoothingStyle::Linear(config::SMOOTHING_LENGTH_MS))
            .with_unit(" dB"),
            log_frequency: nih::FloatParam::new(
                format!("Frequency (Hz){names_suffix}"),
                eq.frequency.log_hz(),
                nih::FloatRange::Linear {
                    min: *log_frequency_range.start(),
                    max: *log_frequency_range.end(),
                },
            )
            .with_smoother(nih::SmoothingStyle::Linear(config::SMOOTHING_LENGTH_MS))
            .with_unit(" Hz")
            .with_value_to_string(sync::Arc::new(
                eq_plotter_egui::utils::log_frequency_to_string,
            ))
            .with_string_to_value(sync::Arc::new(
                eq_plotter_egui::utils::string_to_log_frequency,
            )),
            q: nih::FloatParam::new(
                format!("q{names_suffix}"),
                eq.q,
                nih::FloatRange::Linear {
                    min: *q_range.start(),
                    max: *q_range.end(),
                },
            )
            .with_smoother(nih::SmoothingStyle::Linear(config::SMOOTHING_LENGTH_MS)),
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

    pub fn set_gain_db<F: utils::Float>(&self, gain_db: F, setter: &nih::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.gain_db);
        setter.set_parameter(&self.gain_db, gain_db.to_f32().unwrap());
        setter.end_set_parameter(&self.gain_db);
    }

    pub fn set_log_frequency<F: utils::Float>(
        &self,
        log_frequency: F,
        setter: &nih::ParamSetter<'_>,
    ) {
        setter.begin_set_parameter(&self.log_frequency);
        setter.set_parameter(&self.log_frequency, log_frequency.to_f32().unwrap());
        setter.end_set_parameter(&self.log_frequency);
    }

    pub fn set_q<F: utils::Float>(&self, q: F, setter: &nih::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.q);
        setter.set_parameter(&self.q, q.to_f32().unwrap());
        setter.end_set_parameter(&self.q);
    }

    pub fn set_eq_type(&self, eq_type: eq::EqType, setter: &nih::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.eq_type);
        setter.set_parameter(&self.eq_type, eq_type.into());
        setter.end_set_parameter(&self.eq_type);
    }
}

#[derive(nih::Params)]
pub struct PluginParams {
    #[persist = "editor_state"]
    pub editor_state: sync::Arc<nih_plug_egui::EguiState>,

    #[nested(array, group = "eq_params")]
    pub eq_params: [EqParams; config::NUM_BANDS],

    pub sample_rate: nih::AtomicF32,

    pub show_gain: atomic::AtomicBool,
    pub show_signal_gain_spectrum: atomic::AtomicBool,
    pub show_phase: atomic::AtomicBool,
    pub show_impulse_response: atomic::AtomicBool,
    pub show_poles_and_zeros: atomic::AtomicBool,

    pub selected_eq_index: atomic::AtomicUsize,

    pub analyzer_data: fft::signal_analyzer::SharedData<
        { config::ANALYZER_NUM_BINS },
        { config::MAX_NUM_CHANNELS },
    >,

    pub app_config: app_lib::Config<f64>,
    pub color_palette: colors::ColorPalette,
}

impl PluginParams {
    pub fn eqs<F: utils::Float>(&self) -> [eq::Eq<F>; config::NUM_BANDS] {
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

    pub fn analyzer_coefficients_with_sample_rate(
        sample_rate: f32,
    ) -> fft::signal_analyzer::Coefficients<f32> {
        fft::signal_analyzer::Coefficients {
            sample_rate,
            ..config::DEFAULT_ANALYZER_COEFFICIENTS
        }
    }
}

impl Default for PluginParams {
    fn default() -> Self {
        let app_config = app_lib::Config::<f32>::default();
        let bypassed_init_eq: eq::Eq<f32> =
            eq_plotter_egui::eq_plotter::EqPlotter::BYPASSED_INIT_EQ.into();
        Self {
            editor_state: nih_plug_egui::EguiState::from_size(1000, 700),
            eq_params: std::array::from_fn(|index| {
                EqParams::from_eq(
                    format!(" [{}]", index + 1).as_str(),
                    if index == 0 {
                        &app_config.init_eq()
                    } else {
                        &bypassed_init_eq
                    },
                    app_config.log_frequency_range(),
                    app_config.db_range(),
                    app_config.q_range(),
                )
            }),
            sample_rate: nih::AtomicF32::new(1_f32),
            show_gain: atomic::AtomicBool::new(true),
            show_signal_gain_spectrum: atomic::AtomicBool::new(true),
            show_phase: atomic::AtomicBool::new(false),
            show_impulse_response: atomic::AtomicBool::new(false),
            show_poles_and_zeros: atomic::AtomicBool::new(false),
            selected_eq_index: atomic::AtomicUsize::new(usize::MAX),
            analyzer_data: fft::signal_analyzer::SharedData::new(
                config::DEFAULT_ANALYZER_COEFFICIENTS.sample_rate,
            ),
            app_config: app_lib::Config::default(),
            color_palette: colors::ColorPalette::default(),
        }
    }
}
