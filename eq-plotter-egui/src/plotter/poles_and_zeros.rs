use crate::*;
use audio_lib::biquad;

pub fn add_plot(
    ui: &mut egui::Ui,
    coefficients: &[biquad::coefficients::Coefficients<f64>],
    active_eqs: &[bool],
    plot_size: f32,
    color_palette: &colors::ColorPalette,
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
                        .color(color_palette.eq_stroke[index % color_palette.eq_stroke.len()])
                        .shape(egui_plot::MarkerShape::Cross)
                        .radius(6.0),
                );

                let zeros = biquad::utils::zeros(&c)
                    .iter()
                    .map(|zero| [zero.re, zero.im])
                    .collect::<Vec<_>>();
                plot_ui.points(
                    egui_plot::Points::new("Zeros", zeros)
                        .color(color_palette.eq_stroke[index % color_palette.eq_stroke.len()])
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
