pub mod frequency_transform;
pub mod path;
pub mod plugin_params;

use super::*;
pub use frequency_transform::*;
pub use path::*;
pub use plugin_params::*;

use nice_plug::params::Params;

const SMOOTHING_LENGTH: f32 = 20_f32;
