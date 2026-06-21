use audio_lib::biquad;
use audio_lib::utils;

use crate::*;
use std::sync::{self, atomic};

pub fn create_editor(params: sync::Arc<params::PluginParams>) -> Option<Box<dyn nice::Editor>> {
    let editor_state = params.editor_state.clone();
    let min_size = {
        let state_size = params.editor_state.size();
        egui::Vec2::new(state_size.0 as f32, state_size.1 as f32)
    };
    nice_plug_egui::create_egui_editor(
        params.editor_state.clone(),
        (),
        nice_plug_egui::EguiSettings::default(),
        |egui_ctx, _, _| {
            egui_ctx.set_theme(egui::Theme::Dark);
        },
        move |ui, setter, _, _| {
            if !editor_state.is_open() {
                return;
            }
            nice_plug_egui::resizable_window::ResizableWindow::new("plugin-window")
                .min_size(min_size)
                .show(ui, editor_state.as_ref(), |ui| {
                    // ResizableWindow already has a CentralPanel, so this is a bit weird. But I couldn't find out a better way
                    // to set the global background color.
                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::default()
                                .inner_margin(20)
                                .fill(egui::Color32::from_rgb(32, 35, 38)),
                        )
                        .show_inside(ui, |ui| {
                            let plot_size = ui.available_width().min(0.8 * ui.available_height());
                            let formants = &params.formants;
                            ui.vertical(|ui| {
                                egui::Frame::group(ui.style())
                                    .corner_radius(5)
                                    .show(ui, |ui| {
                                        let plot = egui_plot::Plot::new("Formant Space")
                                            .allow_zoom(false)
                                            .allow_drag(false)
                                            .allow_scroll(false)
                                            .width(plot_size)
                                            .height(plot_size)
                                            .show_background(false)
                                            .show_axes(false)
                                            .show_grid(false)
                                            .label_formatter(|_, point| {
                                                let frequencies = params
                                                    .frequency_transform
                                                    .normalized_to_frequencies([
                                                        point.x as f32,
                                                        point.y as f32,
                                                    ]);
                                                format!(
                                                    "f0: {:.0}\nf1: {:.0}",
                                                    frequencies[0], frequencies[1]
                                                )
                                            });

                                        let f: [f64; 2] = std::array::from_fn(|i| {
                                            formants[i].normalized_frequency.value() as f64
                                        });
                                        let plot_response = plot.show(ui, |plot_ui| {
                                            plot_ui.set_plot_bounds(
                                                egui_plot::PlotBounds::from_min_max(
                                                    [0.0, 0.0],
                                                    [1.0, 1.0],
                                                ),
                                            );

                                            let gain_points = make_gain_response_points(
                                                params.get_biquad_coefficients(),
                                                params.sample_rate.load(atomic::Ordering::Relaxed),
                                            );
                                            plot_ui.line(
                                                egui_plot::Line::new("gain response", gain_points)
                                                    //.color(egui::Color32::from_white_alpha(32)),
                                                    .color(egui::Color32::from_rgba_unmultiplied(
                                                        255, 165, 0, 32,
                                                    )),
                                            );

                                            plot_ui.points(
                                                egui_plot::Points::new(
                                                    "F",
                                                    egui_plot::PlotPoints::from(f),
                                                )
                                                .shape(egui_plot::MarkerShape::Circle)
                                                .color(egui::Color32::ORANGE)
                                                .filled(true)
                                                .radius(5.0),
                                            );
                                            plot_ui.pointer_coordinate()
                                        });
                                        if plot_response.response.is_pointer_button_down_on() {
                                            if let Some(drag_position) = plot_response.inner {
                                                let new_f = [
                                                    drag_position.x as f32,
                                                    drag_position.y as f32,
                                                ];
                                                for i in 0..2 {
                                                    formants[i]
                                                        .set_normalized_frequency(new_f[i], setter);
                                                }
                                            }
                                        }
                                    });
                                for i in 0..2 {
                                    let formant = &formants[i];
                                    ui.horizontal(|ui| {
                                        ui.add(egui::Label::new(format!("F{i} ")));
                                        let mut q = formant.q.value();
                                        let q_response = ui.add(
                                            egui::Slider::new(
                                                &mut q,
                                                range::to_range_inclusive(&formant.q.range()),
                                            )
                                            .prefix("q: "),
                                        );
                                        if q_response.changed() {
                                            formant.set_q(q, setter);
                                        }
                                    });
                                }
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(format!("Dry")));
                                    let mut dry_gain = params.dry_gain_db.value();
                                    let gain_response = ui.add(
                                        egui::Slider::new(
                                            &mut dry_gain,
                                            range::to_range_inclusive(&params.dry_gain_db.range()),
                                        )
                                        .suffix("dB"),
                                    );
                                    if gain_response.changed() {
                                        params.set_dry_gain_db(dry_gain, setter);
                                    }
                                })
                            });
                        });
                });
        },
    )
}

pub fn make_gain_response_points<'a>(
    coefficients: [biquad::coefficients::Coefficients<f32>; 3],
    sample_rate: f32,
) -> egui_plot::PlotPoints<'a> {
    let gain_response =
        utils::make_gain_db_response(biquad::multiband::parallel_sum::make_frequency_response(
            coefficients.into_iter(),
            sample_rate,
        ));
    let log_frequency_range = utils::frequency_to_log(20.0)..=utils::frequency_to_log(20000.0);
    let log_frequency_length = range::get_length(&log_frequency_range);
    let x_to_log_frequency = move |x| log_frequency_range.start() + x as f32 * log_frequency_length;
    let x_to_frequency = move |x| utils::log_to_frequency(x_to_log_frequency(x));
    let gain_db_range = -12_f32..=12_f32;
    let gain_db_length = range::get_length(&gain_db_range);
    let gain_to_normalized = move |gain_db: f32| {
        let gain_clamped = gain_db.clamp(*gain_db_range.start(), *gain_db_range.end());
        ((gain_clamped - gain_db_range.start()) / gain_db_length) as f64
    };
    egui_plot::PlotPoints::from_explicit_callback(
        move |x| gain_to_normalized(gain_response(x_to_frequency(x))),
        0.0..=1.0,
        1000,
    )
}
