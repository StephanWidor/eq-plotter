pub mod params;
pub mod plugin;

pub use nice_plug::prelude as nice;
pub use plugin::Plugin;

nice::nice_export_clap!(Plugin);
nice::nice_export_vst3!(Plugin);
