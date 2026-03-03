use crate::*;
use audio_lib::utils as audio_utils;

pub fn add_plot(
    ui: &mut egui::Ui,
    frequency_responses: &[impl Fn(f64) -> num::Complex<f64>],
    log_frequency_range: &std::ops::RangeInclusive<f64>,
    active_eqs: &[bool],
    multiband_frequency_response: &impl Fn(f64) -> num::Complex<f64>,
    plot_size: f32,
    color_palette: &colors::ColorPalette,
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
                [*log_frequency_range.start(), -std::f64::consts::PI],
                [*log_frequency_range.end(), std::f64::consts::PI],
            ));

            let mut num_active = 0;
            for ((index, response), active) in
                frequency_responses.iter().enumerate().zip(active_eqs)
            {
                if !*active {
                    continue;
                }
                num_active += 1;
                let phase_points = utils::make_log_frequency_points(
                    audio_utils::make_phase_response(response),
                    log_frequency_range,
                );
                plot_ui.line(
                    egui_plot::Line::new("", phase_points)
                        .color(color_palette.eq_stroke[index % color_palette.eq_stroke.len()]),
                );
            }

            if num_active > 1 {
                let phase_points = utils::make_log_frequency_points(
                    audio_utils::make_phase_response(multiband_frequency_response),
                    log_frequency_range,
                );
                plot_ui.line(
                    egui_plot::Line::new("multiband", phase_points)
                        .color(color_palette.multiband_stroke),
                );
            }
        });
}
