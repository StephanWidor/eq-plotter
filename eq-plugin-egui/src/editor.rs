use crate::params;
use app_lib as app;
use nih_plug::prelude as nih;
use std::sync;

pub fn create_editor(
    params: sync::Arc<params::PluginParams>,
    editor_size: (u32, u32),
) -> Option<Box<dyn nih::Editor>> {
    nih_plug_egui::create_egui_editor(
        params.editor_state.clone(),
        (),
        move |egui_ctx, _| {
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                editor_size.0 as f32,
                editor_size.1 as f32,
            )));
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
        },
        move |egui_ctx, setter, _state| {
            egui::CentralPanel::default()
                .frame(
                    egui::Frame::default()
                        .inner_margin(20)
                        .fill(egui::Color32::from_rgb(
                            app::UI_BACKGROUND_COLOR[0],
                            app::UI_BACKGROUND_COLOR[1],
                            app::UI_BACKGROUND_COLOR[2],
                        )),
                )
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
                    );

                    for ((new_eq, old_eq), band_params) in
                        new_eqs.iter().zip(eqs).zip(params.eq_params.as_ref())
                    {
                        if new_eq.gain.db() != old_eq.gain.db() {
                            band_params.set_gain_db(new_eq.gain.db(), setter);
                        }
                        if new_eq.frequency.log_hz() != old_eq.frequency.log_hz() {
                            band_params.set_log_frequency(new_eq.frequency.log_hz(), setter);
                        }
                        if new_eq.q != old_eq.q {
                            band_params.set_q(new_eq.q, setter);
                        }
                        if new_eq.eq_type != old_eq.eq_type {
                            band_params.set_eq_type(new_eq.eq_type, setter);
                        }
                    }
                });
        },
    )
}
