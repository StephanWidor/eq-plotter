use crate::params;
use app_lib as app;
use nih_plug::prelude as nih;
use std::sync;

pub fn create_editor(params: sync::Arc<params::PluginParams>) -> Option<Box<dyn nih::Editor>> {
    let editor_state = params.editor_state.clone();
    nih_plug_egui::create_egui_editor(
        params.editor_state.clone(),
        (),
        |egui_ctx, _| {
            egui_ctx.set_theme(egui::Theme::Dark);
        },
        move |egui_ctx, setter, _state| {
            if !editor_state.is_open() {
                return;
            }
            nih_plug_egui::resizable_window::ResizableWindow::new("resource-window")
                .min_size(egui::Vec2::new(128.0, 128.0))
                .show(egui_ctx, editor_state.as_ref(), |_ui| {
                    // ResizableWindow already has a CentralPanel, so this is a bit weird. But I couldn't find out a better way
                    // to set the global background color.
                    egui::CentralPanel::default()
                        .frame(egui::Frame::default().inner_margin(20).fill(
                            egui::Color32::from_rgb(
                                app::UI_BACKGROUND_COLOR[0],
                                app::UI_BACKGROUND_COLOR[1],
                                app::UI_BACKGROUND_COLOR[2],
                            ),
                        ))
                        .show(egui_ctx, |ui| {
                            let eqs = params.eqs();
                            let mut new_eqs = eqs.clone();
                            eq_plotter_egui::EqPlotter::draw(
                                ui,
                                &mut new_eqs,
                                params
                                    .sample_rate
                                    .load(std::sync::atomic::Ordering::Relaxed)
                                    as f64,
                                &params.show_options,
                            );

                            for ((new_eq, old_eq), band_params) in
                                new_eqs.iter().zip(eqs).zip(params.eq_params.as_ref())
                            {
                                if new_eq.gain.db() != old_eq.gain.db() {
                                    band_params.set_gain_db(new_eq.gain.db(), setter);
                                }
                                if new_eq.frequency.log_hz() != old_eq.frequency.log_hz() {
                                    band_params
                                        .set_log_frequency(new_eq.frequency.log_hz(), setter);
                                }
                                if new_eq.q != old_eq.q {
                                    band_params.set_q(new_eq.q, setter);
                                }
                                if new_eq.eq_type != old_eq.eq_type {
                                    band_params.set_eq_type(new_eq.eq_type, setter);
                                }
                            }
                        });
                });
        },
    )
}
