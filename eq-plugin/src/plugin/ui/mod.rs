pub mod editor;

pub type Settings = crate::ui::settings::Settings<f32>;
pub type Params<const NUM_BANDS: usize> = crate::ui::egui_lib::Params<f32, NUM_BANDS>;
pub use crate::ui::settings::ShowOptions;
