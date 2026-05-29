use crate::*;

pub mod eqs;

pub fn add<F: audio_utils::Float + egui::emath::Numeric, const NUM_BANDS: usize>(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    params: &mut Params<F, NUM_BANDS>,
    spectrum_available: bool,
    eq_ranges: &app_lib::settings::ui::EqRanges<F>,
    eq_colors: &[egui::Color32],
) {
    eqs::add_controls(ui, size, params, spectrum_available, eq_ranges, eq_colors);
}
