use crate::*;
use audio_lib::eq; //use egui::emath;

pub fn add_controls<F: audio_utils::Float + egui::emath::Numeric, const NUM_BANDS: usize>(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    params: &mut Params<F, NUM_BANDS>,
    spectrum_available: bool,
    eq_ranges: &app_lib::settings::ui::EqRanges<F>,
    eq_colors: &[egui::Color32],
) {
    let control_outer_margin = size.x / 25_f32;
    let control_width = size.x - 2_f32 * control_outer_margin;
    let show_options = &mut params.show_options;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .min_scrolled_height(size.y)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    egui::CollapsingHeader::new("Show").show(ui, |ui| {
                        if show_options.gain {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut show_options.gain, "Gain");
                                if spectrum_available {
                                    ui.checkbox(
                                        &mut show_options.signal_gain_spectrum,
                                        "Analyze Signal",
                                    );
                                }
                            });
                        } else {
                            ui.checkbox(&mut show_options.gain, "Gain");
                        }
                        ui.checkbox(&mut show_options.phase, "Phase");
                        ui.checkbox(&mut show_options.impulse_response, "Impulse Response");
                        ui.checkbox(&mut show_options.poles_and_zeros, "Poles And Zeros");
                    });
                    for (index, eq) in params.eqs.iter_mut().enumerate() {
                        add_control(
                            ui,
                            control_width,
                            control_outer_margin,
                            eq_colors[index % eq_colors.len()],
                            eq,
                            eq_ranges,
                        );
                    }
                });
            });
    });
}

fn add_control<F: audio_utils::Float + egui::emath::Numeric>(
    ui: &mut egui::Ui,
    width: f32,
    outer_margin: f32,
    color: egui::Color32,
    eq: &mut eq::Eq<F>,
    eq_ranges: &app_lib::settings::ui::EqRanges<F>,
) {
    let mut gain_db = eq.gain.db();
    let mut log_frequency = eq.frequency.log_hz();
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
                            ui.selectable_value(&mut eq.eq_type, *eq_type, eq_type.to_string());
                        }
                    });

                if eq.eq_type.has_frequency() {
                    ui.add(
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
                    eq.frequency = eq::Frequency::LogHz(log_frequency);
                }

                if eq.eq_type.has_gain_db() {
                    ui.add(
                        egui::Slider::new(&mut gain_db, eq_ranges.db_range.clone())
                            .prefix("gain: ")
                            .suffix("dB"),
                    );
                    eq.gain = eq::Gain::Db(gain_db);
                }

                if eq.eq_type.has_q() {
                    ui.add(egui::Slider::new(&mut eq.q, eq_ranges.q_range.clone()).prefix("Q: "));
                }
            });
        });
}
