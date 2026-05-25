pub mod analyzer;
pub mod config;
pub mod editor;
pub mod params;
pub mod plugin;
pub mod processor;

use config::*;
pub use nice_plug::prelude as nice;

pub type EqRanges = app_lib::settings::ui::EqRanges<f32>;
pub type ImpulseResponseParams = app_lib::settings::ui::ImpulseResponseParams<f32>;
pub type ShowOptions = app_lib::settings::ui::ShowOptions;
pub type Settings<const NUM_BANDS: usize> = app_lib::settings::Settings<f32, NUM_BANDS>;
pub type Plugin =
    plugin::Plugin<{ Config::NUM_BANDS }, { Config::NUM_CHANNELS }, { Config::ANALYZER_NUM_BINS }>;

nice::nice_export_clap!(Plugin);
nice::nice_export_vst3!(Plugin);
