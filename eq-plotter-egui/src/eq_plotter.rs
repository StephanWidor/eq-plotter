use crate::*;
use audio_lib::eq;

pub type EqRanges = app_lib::settings::EqRanges<f64>;
pub type ImpulseResponseSettings = app_lib::settings::ImpulseResponse<f64>;
pub type Settings<const NUM_BANDS: usize> = app_lib::settings::Settings<f64, NUM_BANDS>;

pub struct EqPlotter<const NUM_BANDS: usize> {
    eqs: [eq::Eq<f64>; NUM_BANDS],
    drag_eq_index: usize,
    show_options: options::ShowOptions,
    eq_ranges: EqRanges,
    impulse_response_settings: ImpulseResponseSettings,
    sample_rate: f64,
    color_palette: colors::ColorPalette,
}

impl<const NUM_BANDS: usize> eframe::App for EqPlotter<NUM_BANDS> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .inner_margin(20)
                    .fill(self.color_palette.background),
            )
            .show_inside(ui, |ui| {
                egui_lib::draw(
                    ui,
                    &mut self.eqs,
                    &mut self.drag_eq_index,
                    &self.eq_ranges,
                    &self.impulse_response_settings,
                    self.sample_rate,
                    &mut self.show_options,
                    &self.color_palette,
                );
            });
    }
}

impl<const NUM_BANDS: usize> EqPlotter<NUM_BANDS> {
    pub fn new(settings: &Settings<NUM_BANDS>, color_palette: &colors::ColorPalette) -> Self {
        Self {
            eqs: settings.init_eqs.clone(),
            drag_eq_index: usize::MAX,
            show_options: egui_lib::options::ShowOptions::new_all_enabled(),
            eq_ranges: settings.eq_ranges.clone(),
            impulse_response_settings: settings.impulse_response.clone(),
            sample_rate: settings.init_sample_rate,
            color_palette: color_palette.clone(),
        }
    }
}
