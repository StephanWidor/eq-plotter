use crate::*;

pub fn add_plot(
    ui: &mut egui::Ui,
    impulse_responses: &[Vec<f64>],
    active_eqs: &[bool],
    multiband_impulse_responses: &Vec<f64>,
    plot_size: f32,
    color_palette: &colors::ColorPalette,
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
                        .color(color_palette.eq_stroke[index % color_palette.eq_stroke.len()]),
                );
            }
            if num_active > 1 {
                plot_ui.line(
                    egui_plot::Line::new(
                        "multiband",
                        egui_plot::PlotPoints::from_ys_f64(&multiband_impulse_responses),
                    )
                    .color(color_palette.multiband_stroke),
                );
            }
        });
}
