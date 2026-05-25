use crate::*;
use std::sync::{self, atomic};

pub fn create_editor<
    'a,
    const NUM_BANDS: usize,
    const NUM_CHANNELS: usize,
    const ANALYZER_NUM_BINS: usize,
>(
    params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
) -> Option<Box<dyn nice::Editor>> {
    let editor_state = params.editor_state.clone();
    let min_size = egui::Vec2::new(700.0, 400.0);
    let color_palette = params.color_palette.clone();

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
                                .fill(color_palette.background),
                        )
                        .show_inside(ui, |ui| {
                            let eqs = params.eqs();
                            let mut new_eqs = eqs.clone();
                            let mut show_options = params.show_options();
                            let mut drag_eq_index =
                                params.drag_eq_index.load(atomic::Ordering::Relaxed);
                            let spectrum_gains =
                                params.analyzer_data.linear_gains.consumer.pull_and_read();
                            let spectrum_data = egui_lib::plotter::SpectrumData {
                                frequency_bins: &params
                                    .analyzer_data
                                    .frequency_bins
                                    .read()
                                    .unwrap(),
                                linear_gains: &spectrum_gains,
                            };
                            egui_lib::draw(
                                ui,
                                &mut new_eqs,
                                &mut drag_eq_index,
                                &params.eq_ranges,
                                &params.impulse_response_settings,
                                params.sample_rate.load(atomic::Ordering::Relaxed),
                                &spectrum_data,
                                &mut show_options,
                                &color_palette,
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
                                .drag_eq_index
                                .store(drag_eq_index, atomic::Ordering::Relaxed);
                        });
                });
        },
    )
}
