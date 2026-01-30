use crate::*;
use app_lib as app;
use audio_lib::biquad;
use audio_lib::eq;
use audio_lib::utils as audio_utils;

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

pub fn add_plots(
    ui: &mut egui::Ui,
    available_size: &egui::Vec2,
    eqs: &mut [eq::Eq<f64>],
    selected_eq_index: &mut usize,
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
                            add_gain(
                                ui,
                                eqs,
                                selected_eq_index,
                                &frequency_responses,
                                &multiband_frequency_response,
                                plot_size,
                            );
                        }
                        if show_options.phase {
                            add_phase(
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
                            add_impulse_response(
                                ui,
                                &impulse_responses,
                                &active_eqs,
                                &multiband_impulse_response,
                                plot_size,
                            );
                        }
                        if show_options.poles_and_zeros {
                            add_poles_and_zeros(ui, &coefficients, &active_eqs, plot_size);
                        }
                    });
                });
            });
        });
}

fn add_gain(
    ui: &mut egui::Ui,
    eqs: &mut [eq::Eq<f64>],
    selected_eq_index: &mut usize,
    frequency_responses: &[impl Fn(f64) -> num::Complex<f64>],
    multiband_frequency_response: &impl Fn(f64) -> num::Complex<f64>,
    plot_size: f32,
) {
    assert!(eqs.len() == frequency_responses.len());
    let gain_plot_id = ui.make_persistent_id("gain_plot_id");
    let plot = egui_plot::Plot::new("Gain (dB)")
        .id(gain_plot_id)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .width(plot_size)
        .height(plot_size)
        .auto_bounds([false, false])
        .custom_x_axes(vec![
            egui_plot::AxisHints::new_x()
                .label("Frequency (Hz)")
                .formatter(|_, _| String::new())
                .placement(egui_plot::VPlacement::Top),
            egui_plot::AxisHints::new_x().formatter(utils::log_frequency_formatter),
        ])
        .custom_y_axes(vec![
            egui_plot::AxisHints::new_y()
                .label("Gain (dB)")
                .formatter(|_, _| String::new()),
            egui_plot::AxisHints::new_y().placement(egui_plot::HPlacement::Right),
        ])
        .label_formatter(|_, point| {
            format!(
                "{} Hz, {:.2} dB",
                audio_utils::log_to_frequency(point.x) as i32,
                point.y
            )
        })
        .legend(egui_plot::Legend::default());

    let eq_ids = (0..eqs.len())
        .map(|i| ui.make_persistent_id(format!("eq_id_{}", i)))
        .collect::<Vec<egui::Id>>();
    let eq_id_to_index = |id: egui::Id| eq_ids.iter().position(|eq_id| eq_id.value() == id.value());

    let plot_response = plot.show(ui, |plot_ui| {
        plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
            [app::MIN_LOG_FREQUENCY, app::MIN_GAIN_DB],
            [app::MAX_LOG_FREQUENCY, app::MAX_GAIN_DB],
        ));
        let mut num_active = 0;
        for index in 0..eqs.len() {
            let eq = &eqs[index];
            let eq_id = eq_ids[index];
            let response = &frequency_responses[index];
            if eq.eq_type == eq::EqType::Bypassed {
                continue;
            }
            num_active += 1;
            let gain_points =
                utils::make_log_frequency_points(audio_utils::make_gain_db_response(response));
            plot_ui.line(
                egui_plot::Line::new("", gain_points)
                    .id(eq_id)
                    .color(constants::EQ_COLORS[index % constants::EQ_COLORS.len()]),
            );
        }
        if num_active > 1 {
            let gain_points = utils::make_log_frequency_points(audio_utils::make_gain_db_response(
                multiband_frequency_response,
            ));
            plot_ui.line(
                egui_plot::Line::new("multiband", gain_points)
                    .fill_alpha(0.5)
                    .color(constants::MULTI_BAND_COLOR),
            );
        }
        plot_ui.pointer_coordinate_drag_delta()
    });

    if plot_response.response.is_pointer_button_down_on() {
        if *selected_eq_index >= eqs.len()
            && let Some(hovered_item) = plot_response.hovered_plot_item
        {
            if let Some(hovered_eq_index) = eq_id_to_index(hovered_item) {
                *selected_eq_index = hovered_eq_index;
            }
        }
    } else {
        *selected_eq_index = usize::MAX;
    }

    if *selected_eq_index < eqs.len() {
        let drag_delta = plot_response.inner;
        let eq = &mut eqs[*selected_eq_index];
        eq.frequency = eq::Frequency::LogHz(eq.frequency.log_hz() + drag_delta.x as f64);
        eq.gain = eq::Gain::Db(eq.gain.db() + drag_delta.y as f64);
    }
}

fn add_phase(
    ui: &mut egui::Ui,
    frequency_responses: &[impl Fn(f64) -> num::Complex<f64>],
    active_eqs: &[bool],
    multiband_frequency_response: &impl Fn(f64) -> num::Complex<f64>,
    plot_size: f32,
) {
    egui_plot::Plot::new("Phase")
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .width(plot_size)
        .height(plot_size)
        .auto_bounds([false, false])
        .custom_x_axes(vec![
            egui_plot::AxisHints::new_x()
                .formatter(utils::log_frequency_formatter)
                .placement(egui_plot::VPlacement::Top),
            egui_plot::AxisHints::new_x()
                .label("Frequency (Hz)")
                .formatter(|_, _| String::new()),
        ])
        .custom_y_axes(vec![
            egui_plot::AxisHints::new_y()
                .label("Phase (rad)")
                .formatter(|_, _| String::new()),
            egui_plot::AxisHints::new_y().placement(egui_plot::HPlacement::Right),
        ])
        .label_formatter(|_, point| {
            format!(
                "{} Hz, {:.2} rad",
                audio_utils::log_to_frequency(point.x) as i32,
                point.y
            )
        })
        .show_x(false)
        .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                [app::MIN_LOG_FREQUENCY, -std::f64::consts::PI],
                [app::MAX_LOG_FREQUENCY, std::f64::consts::PI],
            ));

            let mut num_active = 0;
            for ((index, response), active) in
                frequency_responses.iter().enumerate().zip(active_eqs)
            {
                if !*active {
                    continue;
                }
                num_active += 1;
                let phase_points =
                    utils::make_log_frequency_points(audio_utils::make_phase_response(response));
                plot_ui.line(
                    egui_plot::Line::new("", phase_points)
                        .color(constants::EQ_COLORS[index % constants::EQ_COLORS.len()]),
                );
            }

            if num_active > 1 {
                let phase_points = utils::make_log_frequency_points(
                    audio_utils::make_phase_response(multiband_frequency_response),
                );
                plot_ui.line(
                    egui_plot::Line::new("multiband", phase_points)
                        .color(constants::MULTI_BAND_COLOR),
                );
            }
        });
}

fn add_impulse_response(
    ui: &mut egui::Ui,
    impulse_responses: &[Vec<f64>],
    active_eqs: &[bool],
    multiband_impulse_responses: &Vec<f64>,
    plot_size: f32,
) {
    egui_plot::Plot::new("Impulse Response")
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .width(plot_size)
        .height(plot_size)
        .custom_x_axes(vec![
            egui_plot::AxisHints::new_x()
                .label("Samples")
                .formatter(|_, _| String::new())
                .placement(egui_plot::VPlacement::Top),
            egui_plot::AxisHints::new_x(),
        ])
        .custom_y_axes(vec![
            egui_plot::AxisHints::new_y(),
            egui_plot::AxisHints::new_y()
                .label("Impulse Response (Amplitude)")
                .formatter(|_, _| String::new())
                .placement(egui_plot::HPlacement::Right),
        ])
        .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui| {
            let mut num_active = 0;
            for ((index, response), active) in impulse_responses.iter().enumerate().zip(active_eqs)
            {
                if !*active {
                    continue;
                }
                num_active += 1;
                plot_ui.line(
                    egui_plot::Line::new("", egui_plot::PlotPoints::from_ys_f64(&response))
                        .color(constants::EQ_COLORS[index % constants::EQ_COLORS.len()]),
                );
            }
            if num_active > 1 {
                plot_ui.line(
                    egui_plot::Line::new(
                        "multiband",
                        egui_plot::PlotPoints::from_ys_f64(&multiband_impulse_responses),
                    )
                    .color(constants::MULTI_BAND_COLOR),
                );
            }
        });
}

fn add_poles_and_zeros(
    ui: &mut egui::Ui,
    coefficients: &[biquad::coefficients::Coefficients<f64>],
    active_eqs: &[bool],
    plot_size: f32,
) {
    egui_plot::Plot::new("Poles And Zeros")
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .width(plot_size)
        .height(plot_size)
        .custom_x_axes(vec![
            egui_plot::AxisHints::new_x().placement(egui_plot::VPlacement::Top),
            egui_plot::AxisHints::new_x()
                .label("Re")
                .formatter(|_, _| String::new()),
        ])
        .custom_y_axes(vec![
            egui_plot::AxisHints::new_y(),
            egui_plot::AxisHints::new_y()
                .label("Im")
                .formatter(|_, _| String::new())
                .placement(egui_plot::HPlacement::Right),
        ])
        .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui| {
            let mut unstable_biquad = false;
            plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                [-1.5, -1.5],
                [1.5, 1.5],
            ));

            let unit_circle = utils::make_circle_points(1.0, 100);
            plot_ui.polygon(
                egui_plot::Polygon::new("", unit_circle)
                    .width(1_f32)
                    .fill_color(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(1_f32, egui::Color32::GRAY)),
            );

            for ((index, c), active) in coefficients.iter().enumerate().zip(active_eqs) {
                if !*active {
                    continue;
                }
                let poles = biquad::utils::poles(&c)
                    .iter()
                    .map(|pole| [pole.re, pole.im])
                    .collect::<Vec<_>>();
                plot_ui.points(
                    egui_plot::Points::new("Poles", poles)
                        .color(constants::EQ_COLORS[index % constants::EQ_COLORS.len()])
                        .shape(egui_plot::MarkerShape::Cross)
                        .radius(6.0),
                );

                let zeros = biquad::utils::zeros(&c)
                    .iter()
                    .map(|zero| [zero.re, zero.im])
                    .collect::<Vec<_>>();
                plot_ui.points(
                    egui_plot::Points::new("Zeros", zeros)
                        .color(constants::EQ_COLORS[index % constants::EQ_COLORS.len()])
                        .shape(egui_plot::MarkerShape::Circle)
                        .filled(false)
                        .radius(5.0),
                );
                if !biquad::utils::is_stable(c) {
                    unstable_biquad = true;
                }
            }

            if unstable_biquad {
                plot_ui.text(
                    egui_plot::Text::new(
                        "",
                        egui_plot::PlotPoint::new(0.0, 0.5),
                        "Biquad is not stable!",
                    )
                    .color(egui::Color32::RED),
                );
            }
        });
}
