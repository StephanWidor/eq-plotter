#![allow(dead_code)]

pub mod colors;
pub mod control;
pub mod plotter;
pub mod utils;

pub use app_lib::presets;
use audio_lib::utils as audio_utils;
use audio_lib::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(bound = "F: audio_utils::Float")]
pub struct Params<F: audio_utils::Float, const NUM_BANDS: usize> {
    pub show_options: app_lib::settings::ui::ShowOptions,
    #[serde(with = "serde_arrays")]
    pub eqs: [eq::Eq<F>; NUM_BANDS],
    pub sample_rate: F,
    pub drag_eq_index: usize,
    pub preset_selection: presets::Selection,
}

#[derive(Debug, Clone)]
pub struct Settings<F: audio_utils::Float> {
    pub app: app_lib::settings::ui::Settings<F>,
    pub color_palette: colors::ColorPalette,
}

pub struct SpectrumData<'a, F: audio_utils::Float, const NUM_BINS: usize, const NUM_CHANNELS: usize>
{
    pub frequency_bins: &'a fft::LogFrequencyRangeBins<F, NUM_BINS>,
    pub linear_gains: &'a [[F; NUM_BINS]; NUM_CHANNELS],
}

pub fn draw<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_BANDS: usize,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    params: &mut Params<F, NUM_BANDS>,
    presets: &mut presets::Presets<F, NUM_BANDS>,
    settings: &Settings<F>,
    spectrum_data: &Option<SpectrumData<F, NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>>,
) {
    let ui_size = ui.available_size();

    ui.horizontal(|ui| {
        let control_width = 250_f32;
        control::add(
            ui,
            egui::Vec2::new(control_width, ui_size.y),
            params,
            presets,
            settings,
            spectrum_data.is_some(),
        );

        if !(params.show_options.gain
            || params.show_options.phase
            || params.show_options.impulse_response
            || params.show_options.poles_and_zeros)
        {
            return;
        }
        let available_size = egui::Vec2::new(0.96_f32 * (ui_size.x - control_width), ui_size.y);
        plotter::add_plots(ui, &available_size, params, settings, spectrum_data);
    });
}
