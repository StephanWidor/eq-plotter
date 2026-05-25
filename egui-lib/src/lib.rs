#![allow(dead_code)]

pub mod colors;
pub mod control;
pub mod options;
pub mod plotter;
pub mod utils;

use audio_lib::utils as audio_utils;
use audio_lib::*;

pub type EqRanges<F> = app_lib::settings::EqRanges<F>;
pub type ImpulseResponseSettings<F> = app_lib::settings::ImpulseResponse<F>;

#[cfg(not(feature = "analyzer_data"))]
pub fn draw<F: audio_utils::Float + egui::emath::Numeric>(
    ui: &mut egui::Ui,
    eqs: &mut [eq::Eq<F>],
    drag_eq_index: &mut usize,
    eq_ranges: &EqRanges<F>,
    impulse_response_settings: &ImpulseResponseSettings<F>,
    sample_rate: F,
    show_options: &mut options::ShowOptions,
    color_palette: &colors::ColorPalette,
) {
    draw_impl::<F, 0, 0>(
        ui,
        eqs,
        drag_eq_index,
        eq_ranges,
        impulse_response_settings,
        sample_rate,
        show_options,
        color_palette,
    );
}

#[cfg(feature = "analyzer_data")]
pub fn draw<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    eqs: &mut [eq::Eq<F>],
    drag_eq_index: &mut usize,
    eq_ranges: &EqRanges<F>,
    impulse_response_settings: &ImpulseResponseSettings<F>,
    sample_rate: F,
    spectrum_data: &plotter::SpectrumData<F, NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>,
    show_options: &mut options::ShowOptions,
    color_palette: &colors::ColorPalette,
) {
    draw_impl(
        ui,
        eqs,
        drag_eq_index,
        eq_ranges,
        impulse_response_settings,
        sample_rate,
        spectrum_data,
        show_options,
        color_palette,
    );
}

fn draw_impl<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    eqs: &mut [eq::Eq<F>],
    drag_eq_index: &mut usize,
    eq_ranges: &EqRanges<F>,
    impulse_response_settings: &ImpulseResponseSettings<F>,
    sample_rate: F,
    #[cfg(feature = "analyzer_data")] spectrum_data: &plotter::SpectrumData<
        F,
        NUM_SPECTRUM_BINS,
        NUM_SPECTRUM_CHANNELS,
    >,
    show_options: &mut options::ShowOptions,
    color_palette: &colors::ColorPalette,
) {
    let ui_size = ui.available_size();

    ui.horizontal(|ui| {
        let control_width = 250_f32;
        control::add(
            ui,
            egui::Vec2::new(control_width, ui_size.y),
            eqs,
            eq_ranges,
            show_options,
            &color_palette.eq_stroke,
        );

        if !(show_options.gain
            || show_options.phase
            || show_options.impulse_response
            || show_options.poles_and_zeros)
        {
            return;
        }
        let available_size = egui::Vec2::new(0.96_f32 * (ui_size.x - control_width), ui_size.y);
        #[cfg(not(feature = "analyzer_data"))]
        plotter::add_plots(
            ui,
            &available_size,
            eqs,
            drag_eq_index,
            eq_ranges,
            impulse_response_settings,
            sample_rate,
            show_options,
            color_palette,
        );
        #[cfg(feature = "analyzer_data")]
        plotter::add_plots(
            ui,
            &available_size,
            eqs,
            drag_eq_index,
            eq_ranges,
            impulse_response_settings,
            sample_rate,
            spectrum_data,
            show_options,
            color_palette,
        );
    });
}
