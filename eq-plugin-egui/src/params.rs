use app_lib as app;
use audio_lib::*;
use eq_plotter_egui;
use nih_plug::prelude as nih;
use std::sync;

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
    const SMOOTHING_LENGTH_MS: f32 = 20.0;

    fn new(names_suffix: &str) -> Self {
        Self {
            gain_db: nih::FloatParam::new(
                format!("Gain (dB){names_suffix}"),
                app::DEFAULT_EQ.gain.db() as f32,
                nih::FloatRange::Linear {
                    min: app::MIN_GAIN_DB as f32,
                    max: app::MAX_GAIN_DB as f32,
                },
            )
            .with_smoother(nih::SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" dB"),
            log_frequency: nih::FloatParam::new(
                format!("Frequency (Hz){names_suffix}"),
                app::DEFAULT_EQ.frequency.log_hz() as f32,
                nih::FloatRange::Linear {
                    min: app::MIN_LOG_FREQUENCY as f32,
                    max: app::MAX_LOG_FREQUENCY as f32,
                },
            )
            .with_smoother(nih::SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" Hz")
            .with_value_to_string(sync::Arc::new(
                eq_plotter_egui::EqPlotter::log_frequency_to_string,
            ))
            .with_string_to_value(sync::Arc::new(
                eq_plotter_egui::EqPlotter::string_to_log_frequency,
            )),
            q: nih::FloatParam::new(
                format!("q{names_suffix}"),
                app::DEFAULT_EQ.q as f32,
                nih::FloatRange::Linear {
                    min: app::MIN_Q as f32,
                    max: app::MAX_Q as f32,
                },
            )
            .with_smoother(nih::SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS)),
            eq_type: EqTypeParam::new(
                format!("Eq Type{names_suffix}"),
                EqTypeWrapper::from(eq::EqType::Bypassed),
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
    pub eq_params: [EqParams; Self::NUM_BANDS],

    pub sample_rate: nih::AtomicF32,
}

impl PluginParams {
    pub const NUM_BANDS: usize = 8;

    pub fn eqs<F: utils::Float>(&self) -> [eq::Eq<F>; Self::NUM_BANDS] {
        array_init::array_init(|index| self.eq_params[index].to_eq())
    }
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: nih_plug_egui::EguiState::from_size(1000, 700),
            eq_params: array_init::array_init(|index| {
                EqParams::new(format!(" [{}]", index + 1).as_str())
            }),
            sample_rate: nih::AtomicF32::new(1_f32),
        }
    }
}
