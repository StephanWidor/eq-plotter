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
pub struct PluginParams {
    #[persist = "editor-state"]
    pub editor_state: sync::Arc<nih_plug_egui::EguiState>,

    #[id = "gain_db_1"]
    pub gain_db_1: nih::FloatParam,
    #[id = "frequency_1"]
    pub log_frequency_1: nih::FloatParam,
    #[id = "q_1"]
    pub q_1: nih::FloatParam,
    #[id = "eq_type_1"]
    pub eq_type_1: EqTypeParam,

    #[id = "gain_db_2"]
    pub gain_db_2: nih::FloatParam,
    #[id = "frequency_2"]
    pub log_frequency_2: nih::FloatParam,
    #[id = "q_2"]
    pub q_2: nih::FloatParam,
    #[id = "eq_type_2"]
    pub eq_type_2: EqTypeParam,

    #[id = "gain_db_3"]
    pub gain_db_3: nih::FloatParam,
    #[id = "frequency_3"]
    pub log_frequency_3: nih::FloatParam,
    #[id = "q_3"]
    pub q_3: nih::FloatParam,
    #[id = "eq_type_3"]
    pub eq_type_3: EqTypeParam,

    #[id = "gain_db_4"]
    pub gain_db_4: nih::FloatParam,
    #[id = "frequency_4"]
    pub log_frequency_4: nih::FloatParam,
    #[id = "q_4"]
    pub q_4: nih::FloatParam,
    #[id = "eq_type_4"]
    pub eq_type_4: EqTypeParam,

    #[id = "gain_db_5"]
    pub gain_db_5: nih::FloatParam,
    #[id = "frequency_5"]
    pub log_frequency_5: nih::FloatParam,
    #[id = "q_5"]
    pub q_5: nih::FloatParam,
    #[id = "eq_type_5"]
    pub eq_type_5: EqTypeParam,

    pub sample_rate: nih::AtomicF32,
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: nih_plug_egui::EguiState::from_size(
                eq_plotter_egui::EqPlotter::WINDOW_SIZE[0],
                eq_plotter_egui::EqPlotter::WINDOW_SIZE[1],
            ),
            gain_db_1: Self::make_gain_param("gain_db_1"),
            log_frequency_1: Self::make_frequency_param("frequency_1"),
            q_1: Self::make_q_param("q_1"),
            eq_type_1: Self::make_eq_type_param("eq_type_1"),
            gain_db_2: Self::make_gain_param("gain_db_2"),
            log_frequency_2: Self::make_frequency_param("frequency_2"),
            q_2: Self::make_q_param("q_2"),
            eq_type_2: Self::make_eq_type_param("eq_type_2"),
            gain_db_3: Self::make_gain_param("gain_db_3"),
            log_frequency_3: Self::make_frequency_param("frequency_3"),
            q_3: Self::make_q_param("q_3"),
            eq_type_3: Self::make_eq_type_param("eq_type_3"),
            gain_db_4: Self::make_gain_param("gain_db_4"),
            log_frequency_4: Self::make_frequency_param("frequency_4"),
            q_4: Self::make_q_param("q_4"),
            eq_type_4: Self::make_eq_type_param("eq_type_4"),
            gain_db_5: Self::make_gain_param("gain_db_5"),
            log_frequency_5: Self::make_frequency_param("frequency_5"),
            q_5: Self::make_q_param("q_5"),
            eq_type_5: Self::make_eq_type_param("eq_type_5"),

            sample_rate: nih::AtomicF32::new(1_f32),
        }
    }
}

impl PluginParams {
    pub const NUM_BANDS: usize = 5_usize;
    const SMOOTHING_LENGTH_MS: f32 = 20.0;

    fn make_gain_param(name: &str) -> nih::FloatParam {
        nih::FloatParam::new(
            name, //,"gain (dB)",
            app::DEFAULT_EQ.gain.db() as f32,
            nih::FloatRange::Linear {
                min: app::MIN_GAIN_DB as f32,
                max: app::MAX_GAIN_DB as f32,
            },
        )
        .with_smoother(nih::SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
        .with_unit(" dB")
    }

    fn make_frequency_param(name: &str) -> nih::FloatParam {
        nih::FloatParam::new(
            name, //"frequency (Hz)",
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
        ))
    }

    fn make_q_param(name: &str) -> nih::FloatParam {
        nih::FloatParam::new(
            name, //"q",
            app::DEFAULT_EQ.q as f32,
            nih::FloatRange::Linear {
                min: app::MIN_Q as f32,
                max: app::MAX_Q as f32,
            },
        )
        .with_smoother(nih::SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
    }

    fn make_eq_type_param(name: &str) -> EqTypeParam {
        EqTypeParam::new(
            name, //"Eq Type",
            EqTypeWrapper::from(eq::EqType::Bypassed),
        )
    }

    fn to_eq<F: audio_lib::utils::Float>(
        gain_db: f32,
        log_frequency: f32,
        q: f32,
        eq_type: eq::EqType,
    ) -> eq::Eq<F> {
        eq::Eq {
            gain: eq::Gain::Db(F::from(gain_db).unwrap()),
            frequency: eq::Frequency::LogHz(F::from(log_frequency).unwrap()),
            q: F::from(q).unwrap(),
            eq_type: eq_type,
        }
    }

    pub fn eqs<F: utils::Float>(&self) -> [eq::Eq<F>; Self::NUM_BANDS] {
        [
            Self::to_eq(
                self.gain_db_1.value(),
                self.log_frequency_1.value(),
                self.q_1.value(),
                self.eq_type_1.value().into(),
            ),
            Self::to_eq(
                self.gain_db_2.value(),
                self.log_frequency_2.value(),
                self.q_2.value(),
                self.eq_type_2.value().into(),
            ),
            Self::to_eq(
                self.gain_db_3.value(),
                self.log_frequency_3.value(),
                self.q_3.value(),
                self.eq_type_3.value().into(),
            ),
            Self::to_eq(
                self.gain_db_4.value(),
                self.log_frequency_4.value(),
                self.q_4.value(),
                self.eq_type_4.value().into(),
            ),
            Self::to_eq(
                self.gain_db_5.value(),
                self.log_frequency_5.value(),
                self.q_5.value(),
                self.eq_type_5.value().into(),
            ),
        ]
    }

    pub fn update_from_eq(&self, index: usize, eq: &eq::Eq<f64>, setter: &nih::ParamSetter<'_>) {
        let (gain_db, log_frequency, q, eq_type) = match index {
            0 => (
                &self.gain_db_1,
                &self.log_frequency_1,
                &self.q_1,
                &self.eq_type_1,
            ),
            1 => (
                &self.gain_db_2,
                &self.log_frequency_2,
                &self.q_2,
                &self.eq_type_2,
            ),
            2 => (
                &self.gain_db_3,
                &self.log_frequency_3,
                &self.q_3,
                &self.eq_type_3,
            ),
            3 => (
                &self.gain_db_4,
                &self.log_frequency_4,
                &self.q_4,
                &self.eq_type_4,
            ),
            _ => (
                &self.gain_db_5,
                &self.log_frequency_5,
                &self.q_5,
                &self.eq_type_5,
            ),
        };

        setter.begin_set_parameter(gain_db);
        setter.begin_set_parameter(log_frequency);
        setter.begin_set_parameter(q);
        setter.begin_set_parameter(eq_type);
        setter.set_parameter(gain_db, eq.gain.db() as f32);
        setter.set_parameter(log_frequency, eq.frequency.log_hz() as f32);
        setter.set_parameter(q, eq.q as f32);
        setter.set_parameter(eq_type, eq.eq_type.into());
        setter.end_set_parameter(gain_db);
        setter.end_set_parameter(log_frequency);
        setter.end_set_parameter(q);
        setter.end_set_parameter(eq_type);
    }
}
