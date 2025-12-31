use app_lib as app;
use audio_lib::*;
use eq_plotter_egui;
use nih_plug::prelude as nih;
use std::sync;

// hm, can we somehow get rid of this without destroying the nih::Enum and nih::Params derive?
use nih_plug::params::enums::Enum;
use nih_plug::params::Params;

// we have our own EqType here because we need nih-plug's Enum trait (can this be done simpler?)
#[derive(nih::Enum, PartialEq, Clone, Copy)]
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
    pub eq_type: nih::EnumParam<EqType>,

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
            eq_type: nih::EnumParam::new("eq type", app::DEFAULT_EQ.eq_type.into()),
            sample_rate: sync::Arc::new(nih::AtomicF32::new(1.0)),
        }
    }
}
