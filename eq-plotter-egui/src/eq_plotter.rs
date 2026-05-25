use crate::*;
use audio_lib::eq;

pub type Settings<const NUM_BANDS: usize> = app_lib::settings::Settings<f64, NUM_BANDS>;

pub struct EqPlotter<const NUM_BANDS: usize> {
    eqs: [eq::Eq<f64>; NUM_BANDS],
    drag_eq_index: usize,
    settings: Settings<NUM_BANDS>,
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
                    &self.settings.ui.eq_ranges,
                    &self.settings.ui.impulse_response_params,
                    self.settings.init_sample_rate,
                    &mut self.settings.ui.show_options,
                    &self.color_palette,
                );
            });
    }
}

impl<const NUM_BANDS: usize> EqPlotter<NUM_BANDS> {
    pub fn new(settings: Settings<NUM_BANDS>, color_palette: colors::ColorPalette) -> Self {
        Self {
            eqs: settings.init_eqs.clone(),
            drag_eq_index: usize::MAX,
            settings: settings,
            color_palette: color_palette,
        }
    }
}
