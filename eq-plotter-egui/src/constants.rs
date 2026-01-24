use app_lib as app;
use audio_lib::eq;

pub const INIT_WINDOW_SIZE: [u32; 2] = [1250, 1000]; // [width, height]

pub const BACKGROUND_COLOR: egui::Color32 = egui::Color32::from_rgb(
    app::UI_BACKGROUND_COLOR[0],
    app::UI_BACKGROUND_COLOR[1],
    app::UI_BACKGROUND_COLOR[2],
);

pub const EQ_COLORS: [egui::Color32; 8] = [
    egui::Color32::from_rgb(140, 51, 51),
    egui::Color32::from_rgb(140, 107, 54),
    egui::Color32::from_rgb(104, 140, 56),
    egui::Color32::from_rgb(59, 140, 106),
    egui::Color32::from_rgb(59, 102, 140),
    egui::Color32::from_rgb(77, 58, 140),
    egui::Color32::from_rgb(140, 60, 140),
    egui::Color32::from_rgb(140, 82, 99),
];

pub const MULTI_BAND_COLOR: egui::Color32 = egui::Color32::LIGHT_GRAY;

pub const INIT_EQ: eq::Eq<f64> = eq::Eq {
    gain: eq::Gain::Db(3.0),
    frequency: eq::Frequency::Hz(1000.0),
    q: 0.7,
    eq_type: eq::EqType::Bypassed,
};
