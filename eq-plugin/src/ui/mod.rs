pub mod colors;
pub mod editor;
pub mod egui_lib;
pub mod settings;

pub type Params<const NUM_BANDS: usize> = egui_lib::Params<f32, NUM_BANDS>;
pub type Settings = settings::Settings<f32>;
