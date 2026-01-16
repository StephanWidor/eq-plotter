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
                    let eq_params = params.eq_params();
                    eq_plotter_egui::EqPlotter::draw(
                        ui,
                        &mut new_eqs,
                        params
                            .sample_rate
                            .load(std::sync::atomic::Ordering::Relaxed)
                            as f64,
                    );

                    for ((new_eq, old_eq), params) in
                        new_eqs.iter().zip(eqs).zip(eq_params.as_ref())
                    {
                        if !(*new_eq == old_eq) {
                            params.update_from_eq(&new_eq, setter);
                        }
                    }
                });
        },
    )
}
