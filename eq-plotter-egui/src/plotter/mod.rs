mod gain;
mod impulse_response;
mod phase;
mod poles_and_zeros;

use crate::*;
use audio_lib::{biquad, eq};
pub use gain::SpectrumData;

pub fn add_plots<const NUM_SPECTRUM_BINS: usize, const NUM_SPECTRUM_CHANNELS: usize>(
    ui: &mut egui::Ui,
    available_size: &egui::Vec2,
    eqs: &mut [eq::Eq<f64>],
    selected_eq_index: &mut usize,
    spectrum_data: &Option<SpectrumData<NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>>,
    show_options: &options::ShowOptions,
    sample_rate: f64,
) {
    let plot_size = plot_size(show_options, available_size);
    if plot_size < 50_f32 {
        return;
    }

    let active_eqs = eqs
        .iter()
        .map(|eq| eq.eq_type.is_active())
        .collect::<Vec<_>>();
    let coefficients = eqs
        .iter()
        .map(|eq| biquad::coefficients::Coefficients::from_eq(&eq, sample_rate))
        .collect::<Vec<_>>();
    let frequency_responses = coefficients
        .iter()
        .map(|c| biquad::utils::make_frequency_response(&c, sample_rate))
        .collect::<Vec<_>>();
    let multiband_frequency_response =
        biquad::utils::multiband::make_frequency_response(&coefficients, sample_rate);
    egui::Frame::group(ui.style())
        .outer_margin(0_f32)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        if show_options.gain {
                            gain::add_plot(
                                ui,
                                eqs,
                                selected_eq_index,
                                spectrum_data,
                                &frequency_responses,
                                &multiband_frequency_response,
                                plot_size,
                            );
                        }
                        if show_options.phase {
                            phase::add_plot(
                                ui,
                                &frequency_responses,
                                &active_eqs,
                                &multiband_frequency_response,
                                plot_size,
                            );
                        }
                    });

                    ui.vertical(|ui| {
                        if show_options.impulse_response {
                            let (impulse_responses, multiband_impulse_response) =
                                utils::impulse_responses(&coefficients);
                            impulse_response::add_plot(
                                ui,
                                &impulse_responses,
                                &active_eqs,
                                &multiband_impulse_response,
                                plot_size,
                            );
                        }
                        if show_options.poles_and_zeros {
                            poles_and_zeros::add_plot(ui, &coefficients, &active_eqs, plot_size);
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
