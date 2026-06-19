pub mod config;
pub mod params;
pub mod plugin;
pub mod presets;
pub mod processing;
pub mod settings;
pub mod ui;

use config::*;
pub use nice_plug::prelude as nice;

pub type EqRanges = crate::ui::settings::EqRanges<f32>;
pub type ImpulseResponseParams = crate::ui::settings::ImpulseResponseParams<f32>;
pub type ShowOptions = crate::ui::settings::ShowOptions;
pub type AppSettings<const NUM_BANDS: usize> = settings::Settings<f32, NUM_BANDS>;
pub type Presets<const NUM_BANDS: usize> = presets::Presets<f32, NUM_BANDS>;
pub type Plugin =
    plugin::Plugin<{ Config::NUM_BANDS }, { Config::NUM_CHANNELS }, { Config::ANALYZER_NUM_BINS }>;

nice::nice_export_clap!(Plugin);
nice::nice_export_vst3!(Plugin);
