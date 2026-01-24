use crate::*;
use app_lib as app;
use audio_lib::eq;

pub fn add_eq_controls(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    eqs: &mut [eq::Eq<f64>],
    show_options: &mut options::ShowOptions,
) {
    let control_outer_margin = size.x / 25_f32;
    let control_width = size.x - 2_f32 * control_outer_margin;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .min_scrolled_height(size.y)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    egui::CollapsingHeader::new("Show").show(ui, |ui| {
                        ui.checkbox(&mut show_options.gain, "Gain");
                        ui.checkbox(&mut show_options.phase, "Phase");
                        ui.checkbox(&mut show_options.impulse_response, "Impulse Response");
                        ui.checkbox(&mut show_options.poles_and_zeros, "Poles And Zeros");
                    });
                    for (index, eq) in eqs.iter_mut().enumerate() {
                        add_eq_control(
                            ui,
                            control_width,
                            control_outer_margin,
                            constants::EQ_COLORS[index % constants::EQ_COLORS.len()],
                            eq,
                        );
                    }
                });
            });
    });
}

fn add_eq_control(
    ui: &mut egui::Ui,
    width: f32,
    outer_margin: f32,
    color: egui::Color32,
    eq: &mut eq::Eq<f64>,
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
                            app::MIN_LOG_FREQUENCY..=app::MAX_LOG_FREQUENCY,
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
                        egui::Slider::new(&mut gain_db, app::MIN_GAIN_DB..=app::MAX_GAIN_DB)
                            .prefix("gain: ")
                            .suffix("dB"),
                    );
                    eq.gain = eq::Gain::Db(gain_db);
                }

                if eq.eq_type.has_q() {
                    ui.add(egui::Slider::new(&mut eq.q, app::MIN_Q..=app::MAX_Q).prefix("Q: "));
                }
            });
        });
}
