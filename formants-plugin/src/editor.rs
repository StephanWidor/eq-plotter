use crate::*;
use std::sync;

pub fn create_editor(params: sync::Arc<params::PluginParams>) -> Option<Box<dyn nice::Editor>> {
    let editor_state = params.editor_state.clone();
    let min_size = egui::Vec2::new(400.0, 400.0);

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
                            let plot_size = ui.available_height().min(ui.available_width());
                            let plot = egui_plot::Plot::new("Formant Space")
                                .allow_zoom(false)
                                .allow_drag(false)
                                .allow_scroll(false)
                                .width(plot_size)
                                .height(plot_size);

                            let f: [f64; 2] = std::array::from_fn(|i| {
                                params.formants[i].normalized_frequency.value() as f64
                            });
                            let plot_response = plot.show(ui, |plot_ui| {
                                plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                                    [0.0, 0.0],
                                    [1.0, 1.0],
                                ));

                                plot_ui.points(
                                    egui_plot::Points::new("F", egui_plot::PlotPoints::from(f))
                                        .shape(egui_plot::MarkerShape::Circle)
                                        .color(egui::Color32::ORANGE)
                                        .filled(true)
                                        .radius(5.0),
                                );
                                plot_ui.pointer_coordinate()
                            });
                            if plot_response.response.is_pointer_button_down_on() {
                                if let Some(drag_position) = plot_response.inner {
                                    let new_f = [drag_position.x as f32, drag_position.y as f32];
                                    for i in 0..2 {
                                        setter.begin_set_parameter(
                                            &params.formants[i].normalized_frequency,
                                        );
                                        setter.set_parameter(
                                            &params.formants[i].normalized_frequency,
                                            (new_f[i]).clamp(0_f32, 1_f32),
                                        );
                                        setter.end_set_parameter(
                                            &params.formants[i].normalized_frequency,
                                        );
                                    }
                                }
                            }
                        });
                });
        },
    )
}
