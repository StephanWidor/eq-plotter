mod gain;
mod impulse_response;
mod phase;
mod poles_and_zeros;

use crate::*;
use audio_lib::{biquad, eq};

pub fn add_plots<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_BANDS: usize,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    available_size: &egui::Vec2,
    params: &mut Params<F, NUM_BANDS>,
    settings: &Settings<F>,
    spectrum_data: &Option<SpectrumData<F, NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>>,
) {
    let show_options = &mut params.show_options;
    let plot_size = plot_size(show_options, available_size);
    if plot_size < 50_f32 {
        return;
    }
    let drag_eq_index = &mut params.drag_eq_index;
    let sample_rate = params.sample_rate;
    let coefficients = params
        .eqs
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
                                    *drag_eq_index,
                                    &settings.app.eq_ranges,
                                    spectrum_data,
                                    plot_size,
                                    &settings.color_palette,
                                );
                            *drag_eq_index = indexed_eq_diff.index;
                            if let Some(eq_diff) = indexed_eq_diff.diff {
                                let eq = &mut params.eqs[*drag_eq_index];
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
                                &settings.app.eq_ranges.log_frequency_range,
                                plot_size,
                                &settings.color_palette,
                            );
                        }
                    });

                    ui.vertical(|ui| {
                        if show_options.impulse_response {
                            impulse_response::add_plot(
                                ui,
                                &coefficients,
                                &settings.app.impulse_response_params,
                                plot_size,
                                &settings.color_palette,
                            );
                        }
                        if show_options.poles_and_zeros {
                            poles_and_zeros::add_plot(
                                ui,
                                &coefficients,
                                plot_size,
                                &settings.color_palette,
                            );
                        }
                    });
                });
            });
        });
}

fn plot_size(
    show_options: &app_lib::settings::ui::ShowOptions,
    available_size: &egui::Vec2,
) -> f32 {
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
