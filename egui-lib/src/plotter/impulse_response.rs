use crate::*;

pub fn add_plot<F: audio_utils::Float + egui::emath::Numeric>(
    ui: &mut egui::Ui,
    coefficients: &[Option<biquad::coefficients::Coefficients<F>>],
    impulse_response_params: &ImpulseResponseParams<F>,
    plot_size: f32,
    color_palette: &colors::ColorPalette,
) {
    let to_plot_points = |impulse_response: Vec<F>| {
        egui_plot::PlotPoints::new(
            impulse_response
                .iter()
                .enumerate()
                .map(|(i, &y)| [i as f64, y.to_f64()])
                .collect(),
        )
    };

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
            let active_coefficients = coefficients.iter().filter(|c| c.is_some());
            if active_coefficients.clone().take(2).count() > 1 {
                let impulse_response = biquad::utils::multiband::impulse_response_for_coefficients(
                    active_coefficients.map(|c| c.as_ref().unwrap().clone()),
                    impulse_response_params.eps,
                    impulse_response_params.hold_length,
                    impulse_response_params.max_length,
                );
                plot_ui.line(
                    egui_plot::Line::new("multiband", to_plot_points(impulse_response))
                        .color(color_palette.multiband_stroke),
                );
            }
            for (index, c) in coefficients.iter().enumerate() {
                if let Some(c) = c {
                    let impulse_response = biquad::utils::impulse_response_for_coefficients(
                        c.clone(),
                        impulse_response_params.eps,
                        impulse_response_params.hold_length,
                        impulse_response_params.max_length,
                    );
                    plot_ui.line(
                        egui_plot::Line::new("", to_plot_points(impulse_response))
                            .color(color_palette.eq_stroke[index % color_palette.eq_stroke.len()]),
                    );
                }
            }
        });
}
