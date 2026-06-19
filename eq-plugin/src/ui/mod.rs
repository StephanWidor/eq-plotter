pub mod editor;
pub mod egui_lib;

pub type Params<const NUM_BANDS: usize> = egui_lib::Params<f32, NUM_BANDS>;
pub type Settings = egui_lib::Settings<f32>;
