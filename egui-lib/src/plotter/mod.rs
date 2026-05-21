mod gain;
mod impulse_response;
mod phase;
mod poles_and_zeros;

use crate::*;
use audio_lib::{biquad, eq};

#[cfg(not(feature = "analyzer_data"))]
pub fn add_plots<F: audio_utils::Float + egui::emath::Numeric>(
    ui: &mut egui::Ui,
    available_size: &egui::Vec2,
    eqs: &mut [eq::Eq<F>],
    selected_eq_index: &mut usize,
    eq_ranges: &EqRanges<F>,
    impulse_response_settings: &ImpulseResponseSettings<F>,
    sample_rate: F,
    show_options: &options::ShowOptions,
    color_palette: &colors::ColorPalette,
) {
    add_plots_impl::<F, 0, 0>(
        ui,
        available_size,
        eqs,
        selected_eq_index,
        eq_ranges,
        impulse_response_settings,
        sample_rate,
        show_options,
        color_palette,
    );
}

#[cfg(feature = "analyzer_data")]
pub struct SpectrumData<'a, F: audio_utils::Float, const NUM_BINS: usize, const NUM_CHANNELS: usize>
{
    pub frequency_bins: &'a fft::LogFrequencyRangeBins<F, NUM_BINS>,
    pub linear_gains: &'a [[F; NUM_BINS]; NUM_CHANNELS],
}

#[cfg(feature = "analyzer_data")]
pub fn add_plots<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    available_size: &egui::Vec2,
    eqs: &mut [eq::Eq<F>],
    selected_eq_index: &mut usize,
    eq_ranges: &EqRanges<F>,
    impulse_response_settings: &ImpulseResponseSettings<F>,
    sample_rate: F,
    spectrum_data: &SpectrumData<F, NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>,
    show_options: &options::ShowOptions,
    color_palette: &colors::ColorPalette,
) {
    add_plots_impl(
        ui,
        available_size,
        eqs,
        selected_eq_index,
        eq_ranges,
        impulse_response_settings,
        sample_rate,
        spectrum_data,
        show_options,
        color_palette,
    );
}

fn add_plots_impl<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    available_size: &egui::Vec2,
    eqs: &mut [eq::Eq<F>],
    selected_eq_index: &mut usize,
    eq_ranges: &EqRanges<F>,
    impulse_response_settings: &ImpulseResponseSettings<F>,
    sample_rate: F,
    #[cfg(feature = "analyzer_data")] spectrum_data: &SpectrumData<
        F,
        NUM_SPECTRUM_BINS,
        NUM_SPECTRUM_CHANNELS,
    >,
    show_options: &options::ShowOptions,
    color_palette: &colors::ColorPalette,
) {
    let plot_size = plot_size(show_options, available_size);
    if plot_size < 50_f32 {
        return;
    }

    let coefficients = eqs
        .iter()
        .map(|eq| {
            if eq.eq_type.is_active() {
                Some(biquad::coefficients::Coefficients::from_eq(
                    &eq,
                    sample_rate,
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    egui::Frame::group(ui.style())
        .outer_margin(0_f32)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        if show_options.gain {
                            let indexed_eq_diff =
                                gain::add_plot::<F, NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>(
                                    ui,
                                    &coefficients,
                                    sample_rate,
                                    *selected_eq_index,
                                    eq_ranges,
                                    #[cfg(feature = "analyzer_data")]
                                    spectrum_data,
                                    plot_size,
                                    color_palette,
                                );
                            *selected_eq_index = indexed_eq_diff.index;
                            if let Some(eq_diff) = indexed_eq_diff.diff {
                                let eq = &mut eqs[*selected_eq_index];
                                eq.frequency = eq::Frequency::LogHz(
                                    eq.frequency.log_hz() + eq_diff.log_frequency,
                                );
                                eq.gain = eq::Gain::Db(eq.gain.db() + eq_diff.gain_db);
                            }
                        }
                        if show_options.phase {
                            phase::add_plot(
                                ui,
                                &coefficients,
                                sample_rate,
                                &eq_ranges.log_frequency_range,
                                plot_size,
                                color_palette,
                            );
                        }
                    });

                    ui.vertical(|ui| {
                        if show_options.impulse_response {
                            impulse_response::add_plot(
                                ui,
                                &coefficients,
                                impulse_response_settings,
                                plot_size,
                                color_palette,
                            );
                        }
                        if show_options.poles_and_zeros {
                            poles_and_zeros::add_plot(ui, &coefficients, plot_size, color_palette);
                        }
                    });
                });
            });
        });
}

fn plot_size(show_options: &options::ShowOptions, available_size: &egui::Vec2) -> f32 {
    let num_rows = (((show_options.gain && show_options.phase)
        || (show_options.impulse_response && show_options.poles_and_zeros))
        as usize
        + 1) as f32;
    let num_columns = (((show_options.gain || show_options.phase)
        && (show_options.impulse_response || show_options.poles_and_zeros))
        as usize
        + 1) as f32;
    (available_size.x / num_columns).min(available_size.y / num_rows) - 15_f32
}
