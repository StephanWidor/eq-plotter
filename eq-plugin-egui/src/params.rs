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

    #[id = "gain_db"]
    pub gain_db: nih::FloatParam,

    #[id = "frequency"]
    pub log_frequency: nih::FloatParam,

    #[id = "q"]
    pub q: nih::FloatParam,

    #[id = "eq_type"]
    pub eq_type: EqTypeParam,

    pub sample_rate: sync::Arc<nih::AtomicF32>,
}

impl PluginParams {
    const SMOOTHING_LENGTH_MS: f32 = 20.0;
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: nih_plug_egui::EguiState::from_size(
                eq_plotter_egui::EqPlotter::WINDOW_SIZE[0],
                eq_plotter_egui::EqPlotter::WINDOW_SIZE[1],
            ),
            gain_db: nih::FloatParam::new(
                "gain (dB)",
                app::DEFAULT_EQ.gain.db() as f32,
                nih::FloatRange::Linear {
                    min: app::MIN_GAIN_DB as f32,
                    max: app::MAX_GAIN_DB as f32,
                },
            )
            .with_smoother(nih::SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" dB"),
            log_frequency: nih::FloatParam::new(
                "frequency (Hz)",
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
                "q",
                app::DEFAULT_EQ.q as f32,
                nih::FloatRange::Linear {
                    min: app::MIN_Q as f32,
                    max: app::MAX_Q as f32,
                },
            )
            .with_smoother(nih::SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS)),
            eq_type: EqTypeParam::new(
                app::DEFAULT_EQ.eq_type.to_string(),
                EqTypeWrapper::from(app::DEFAULT_EQ.eq_type),
            ),
            sample_rate: sync::Arc::new(nih::AtomicF32::new(1.0)),
        }
    }
}
