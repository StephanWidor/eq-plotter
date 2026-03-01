use crate::*;
use app_lib as app;
use audio_lib::utils as audio_utils;

pub fn add_plot(
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
