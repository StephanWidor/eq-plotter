use crate::params;
use app_lib as app;
use audio_lib::eq;
use nih_plug::prelude as nih;
use std::sync::Arc;

pub fn create_editor(
    params: Arc<params::PluginParams>,
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
                    let eq = eq::Eq {
                        gain: eq::Gain::Db(params.gain_db.value() as f64),
                        frequency: eq::Frequency::LogHz(params.log_frequency.value() as f64),
                        q: params.q.value() as f64,
                        eq_type: params.eq_type.value().into(),
                    };
                    let mut new_eqs = [eq.clone()];
                    eq_plotter_egui::EqPlotter::draw(
                        ui,
                        &mut new_eqs,
                        params
                            .sample_rate
                            .load(std::sync::atomic::Ordering::Relaxed)
                            as f64,
                    );

                    let new_eq = new_eqs[0];
                    if eq == new_eq {
                        return; // no changes
                    }

                    setter.begin_set_parameter(&params.gain_db);
                    setter.begin_set_parameter(&params.log_frequency);
                    setter.begin_set_parameter(&params.q);
                    setter.begin_set_parameter(&params.eq_type);
                    setter.set_parameter(&params.gain_db, new_eq.gain.db() as f32);
                    setter.set_parameter(&params.log_frequency, new_eq.frequency.log_hz() as f32);
                    setter.set_parameter(&params.q, new_eq.q as f32);
                    setter.set_parameter(&params.eq_type, new_eq.eq_type.into());
                    setter.end_set_parameter(&params.gain_db);
                    setter.end_set_parameter(&params.log_frequency);
                    setter.end_set_parameter(&params.q);
                    setter.end_set_parameter(&params.eq_type);
                });
        },
    )
}
