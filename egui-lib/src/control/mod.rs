use crate::*;
use audio_lib::eq;
pub mod eqs;

pub fn add<F: audio_utils::Float + egui::emath::Numeric>(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    eqs: &mut [eq::Eq<F>],
    eq_ranges: &EqRanges<F>,
    show_options: &mut options::ShowOptions,
    eq_colors: &[egui::Color32],
) {
    eqs::add_controls(ui, size, eqs, &eq_ranges, show_options, eq_colors);
}
