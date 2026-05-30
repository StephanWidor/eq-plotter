use crate::*;
use audio_lib::eq;
use egui::accesskit::Uuid;

/// Return true if the eq has changed
pub fn add_slider_controls<F: audio_utils::Float + egui::emath::Numeric>(
    ui: &mut egui::Ui,
    width: f32,
    outer_margin: f32,
    color: egui::Color32,
    eq_ranges: &app_lib::settings::ui::EqRanges<F>,
    eq: &mut eq::Eq<F>,
) -> bool {
    let mut gain_db = eq.gain.db();
    let mut log_frequency = eq.frequency.log_hz();
    let mut eq_has_changed = false;
    egui::Frame::group(ui.style())
        .fill(color)
        .multiply_with_opacity(0.2_f32)
        .corner_radius(5)
        .outer_margin(outer_margin)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                egui::ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(eq.eq_type.to_string())
                    .width(width)
                    .show_ui(ui, |ui| {
                        for eq_type in eq::EqType::ALL.iter() {
                            let response =
                                ui.selectable_value(&mut eq.eq_type, *eq_type, eq_type.to_string());
                            eq_has_changed |= response.changed();
                        }
                    });

                if eq.eq_type.has_frequency() {
                    let response = ui.add(
                        egui::Slider::new(
                            &mut log_frequency,
                            eq_ranges.log_frequency_range.clone(),
                        )
                        .custom_formatter(|log_frequency, _| {
                            utils::log_frequency_to_string(log_frequency)
                        })
                        .custom_parser(utils::string_to_log_frequency)
                        .prefix("frequency: ")
                        .suffix("Hz"),
                    );
                    if response.changed() {
                        eq.frequency = eq::Frequency::LogHz(log_frequency);
                        eq_has_changed = true;
                    }
                }

                if eq.eq_type.has_gain_db() {
                    let response = ui.add(
                        egui::Slider::new(&mut gain_db, eq_ranges.db_range.clone())
                            .prefix("gain: ")
                            .suffix("dB"),
                    );
                    if response.changed() {
                        eq.gain = eq::Gain::Db(gain_db);
                        eq_has_changed = true;
                    }
                }

                if eq.eq_type.has_q() {
                    let response = ui
                        .add(egui::Slider::new(&mut eq.q, eq_ranges.q_range.clone()).prefix("Q: "));
                    if response.changed() {
                        eq_has_changed = true;
                    }
                }
            });
        });
    eq_has_changed
}

pub fn add_preset_controls<F: audio_utils::Float, const NUM_BANDS: usize>(
    ui: &mut egui::Ui,
    width: f32,
    outer_margin: f32,
    params: &mut Params<F, NUM_BANDS>,
    presets: &mut app_lib::presets::Presets<F, NUM_BANDS>,
) {
    egui::Frame::group(ui.style())
        .corner_radius(5)
        .outer_margin(outer_margin)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let no_preset_text = || String::from("No preset chosen");
                let mut selected_preset_name = match params.preset_selection.clone() {
                    presets::Selection::None => no_preset_text(),
                    presets::Selection::Selected(name) => {
                        if presets.contains(&name) {
                            name.clone()
                        } else {
                            params.preset_selection = presets::Selection::None;
                            no_preset_text()
                        }
                    }
                    presets::Selection::SelectedChanged(name) => {
                        if presets.contains(&name) {
                            name.clone() + "*"
                        } else {
                            params.preset_selection = presets::Selection::None;
                            no_preset_text()
                        }
                    }
                };

                let mut selection_changed = false;
                egui::ComboBox::from_id_salt("preset_selector")
                    .selected_text(selected_preset_name.clone())
                    .width(width)
                    .show_ui(ui, |ui| {
                        for preset_name in presets.names_iter() {
                            let response = ui.selectable_value(
                                &mut selected_preset_name,
                                preset_name.clone(),
                                preset_name.as_str(),
                            );
                            selection_changed |= response.changed();
                        }
                    });
                if selection_changed {
                    presets.get_inline(&selected_preset_name, &mut params.eqs);
                    params.preset_selection = presets::Selection::Selected(selected_preset_name);
                }

                match params.preset_selection.clone() {
                    presets::Selection::None => {
                        if ui.button("Add").clicked() {
                            // TODO: preliminary
                            let name = Uuid::new_v4().to_string();
                            if presets.add(name.clone(), params.eqs.clone()) {
                                params.preset_selection = presets::Selection::Selected(name);
                            }
                        }
                    }
                    presets::Selection::Selected(name) => {
                        if ui.button("Delete").clicked() {
                            presets.remove(&name);
                            params.preset_selection = presets::Selection::None;
                        }
                    }
                    presets::Selection::SelectedChanged(name) => {
                        ui.horizontal(|ui| {
                            if ui.button("Add").clicked() {
                                // TODO: preliminary
                                let name = Uuid::new_v4().to_string();
                                if presets.add(name.clone(), params.eqs.clone()) {
                                    params.preset_selection = presets::Selection::Selected(name);
                                }
                            }
                            if ui.button("Save").clicked() {
                                presets.force_add(name.clone(), params.eqs.clone());
                                params.preset_selection =
                                    presets::Selection::Selected(name.clone());
                            }
                        });
                    }
                };
            });
        });
}
