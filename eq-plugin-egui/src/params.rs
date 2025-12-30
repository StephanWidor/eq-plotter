use app_lib as app;
use audio_lib::*;
use eq_plotter_egui;
use nih_plug::prelude::*;
use std::sync;

// we have our own EqType here because we need nih-plugs Enum trait (can this be done simpler?)
#[derive(Enum, PartialEq, Clone, Copy)]
pub enum EqType {
    Volume,
    LowPass,
    HighPass,
    BandPass,
    AllPass,
    Notch,
    Peak,
    LowShelf,
    HighShelf,
}

impl From<eq::EqType> for EqType {
    fn from(eq: eq::EqType) -> Self {
        match eq {
            eq::EqType::Volume => EqType::Volume,
            eq::EqType::LowPass => EqType::LowPass,
            eq::EqType::HighPass => EqType::HighPass,
            eq::EqType::BandPass => EqType::BandPass,
            eq::EqType::AllPass => EqType::AllPass,
            eq::EqType::Notch => EqType::Notch,
            eq::EqType::Peak => EqType::Peak,
            eq::EqType::LowShelf => EqType::LowShelf,
            eq::EqType::HighShelf => EqType::HighShelf,
        }
    }
}
impl Into<eq::EqType> for EqType {
    fn into(self) -> eq::EqType {
        match self {
            EqType::Volume => eq::EqType::Volume,
            EqType::LowPass => eq::EqType::LowPass,
            EqType::HighPass => eq::EqType::HighPass,
            EqType::BandPass => eq::EqType::BandPass,
            EqType::AllPass => eq::EqType::AllPass,
            EqType::Notch => eq::EqType::Notch,
            EqType::Peak => eq::EqType::Peak,
            EqType::LowShelf => eq::EqType::LowShelf,
            EqType::HighShelf => eq::EqType::HighShelf,
        }
    }
}

#[derive(Params)]
pub struct EqPluginParams {
    #[persist = "editor-state"]
    pub editor_state: sync::Arc<nih_plug_egui::EguiState>,

    #[id = "gain_db"]
    pub gain_db: FloatParam,

    #[id = "frequency"]
    pub log_frequency: FloatParam,

    #[id = "q"]
    pub q: FloatParam,

    #[id = "eq_type"]
    pub eq_type: EnumParam<EqType>,
}

impl EqPluginParams {
    const SMOOTHING_LENGTH_MS: f32 = 20.0;
}

impl Default for EqPluginParams {
    fn default() -> Self {
        Self {
            editor_state: nih_plug_egui::EguiState::from_size(
                eq_plotter_egui::EqPlotter::WINDOW_SIZE[0],
                eq_plotter_egui::EqPlotter::WINDOW_SIZE[1],
            ),
            gain_db: FloatParam::new(
                "gain (dB)",
                app::DEFAULT_EQ.gain.db() as f32,
                FloatRange::Linear {
                    min: app::MIN_GAIN_DB as f32,
                    max: app::MAX_GAIN_DB as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" dB"),
            log_frequency: FloatParam::new(
                "frequency (Hz)",
                app::DEFAULT_EQ.frequency.log_hz() as f32,
                FloatRange::Linear {
                    min: app::MIN_LOG_FREQUENCY as f32,
                    max: app::MAX_LOG_FREQUENCY as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS))
            .with_unit(" Hz")
            .with_value_to_string(sync::Arc::new(
                eq_plotter_egui::EqPlotter::log_frequency_to_string,
            ))
            .with_string_to_value(sync::Arc::new(
                eq_plotter_egui::EqPlotter::string_to_log_frequency,
            )),
            q: FloatParam::new(
                "q",
                app::DEFAULT_EQ.q as f32,
                FloatRange::Linear {
                    min: app::MIN_Q as f32,
                    max: app::MAX_Q as f32,
                },
            )
            .with_smoother(SmoothingStyle::Linear(Self::SMOOTHING_LENGTH_MS)),
            eq_type: EnumParam::new("eq type", app::DEFAULT_EQ.eq_type.into()),
        }
    }
}
