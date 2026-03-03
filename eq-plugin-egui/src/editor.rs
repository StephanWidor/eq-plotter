use crate::*;
use eq_plotter_egui::*;
use nih_plug::prelude as nih;
use std::sync::{self, atomic};

pub fn create_editor<'a>(params: sync::Arc<params::PluginParams>) -> Option<Box<dyn nih::Editor>> {
    let editor_state = params.editor_state.clone();
    let app_config = &params.app_config;
    let log_frequency_range = app_config.log_frequency_range().clone();
    let db_range = app_config.db_range().clone();
    let q_range = app_config.q_range().clone();
    let min_size = egui::Vec2::new(700.0, 400.0);
    let color_palette = params.color_palette.clone();
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
            nih_plug_egui::resizable_window::ResizableWindow::new("plugin-window")
                .min_size(min_size)
                .show(egui_ctx, editor_state.as_ref(), |_ui| {
                    // ResizableWindow already has a CentralPanel, so this is a bit weird. But I couldn't find out a better way
                    // to set the global background color.
                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::default()
                                .inner_margin(20)
                                .fill(color_palette.background),
                        )
                        .show(egui_ctx, |ui| {
                            let eqs = params.eqs();
                            let mut new_eqs = eqs.clone();
                            let mut show_options = params.show_options();
                            let mut selected_eq_index =
                                params.selected_eq_index.load(atomic::Ordering::Relaxed);
                            let spectrum_gains =
                                params.analyzer_data.linear_gains.consumer.pull_and_read();
                            let spectrum_data = Option::Some(eq_plotter::SpectrumData {
                                frequency_bins: &params
                                    .analyzer_data
                                    .frequency_bins
                                    .read()
                                    .unwrap(),
                                linear_gains: &spectrum_gains,
                            });
                            eq_plotter::draw(
                                ui,
                                &mut new_eqs,
                                &mut selected_eq_index,
                                &log_frequency_range,
                                &db_range,
                                &q_range,
                                &spectrum_data,
                                &mut show_options,
                                &color_palette,
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

                            params.set_show_options(&show_options);
                            params
                                .selected_eq_index
                                .store(selected_eq_index, atomic::Ordering::Relaxed);
                        });
                });
        },
    )
}
