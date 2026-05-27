use crate::*;
use audio_lib::{eq, persistence};

pub type Settings<const NUM_BANDS: usize> = app_lib::settings::Settings<f64, NUM_BANDS>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MultibandEq<const NUM_BANDS: usize> {
    #[serde(with = "serde_arrays")]
    pub eqs: [eq::Eq<f64>; NUM_BANDS],
}

pub struct EqPlotter<const NUM_BANDS: usize> {
    eqs: MultibandEq<NUM_BANDS>,
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
                    &mut self.eqs.eqs,
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
        let eqs = if let Some(eqs) = persistence::create_from_json_file::<MultibandEq<NUM_BANDS>>(
            &settings.persistence_dir.join("eqs.json").as_path(),
        ) {
            eqs
        } else {
            MultibandEq {
                eqs: settings.init_eqs.clone(),
            }
        };
        let mut eq_plotter = Self {
            eqs: eqs,
            drag_eq_index: usize::MAX,
            settings: settings,
            color_palette: color_palette,
        };
        if let Some(show_options) = persistence::create_from_json_file::<ShowOptions>(
            &eq_plotter
                .settings
                .persistence_dir
                .join("show_options.json")
                .as_path(),
        ) {
            eq_plotter.settings.ui.show_options = show_options.clone();
        }
        eq_plotter
    }
}

impl<const NUM_BANDS: usize> Drop for EqPlotter<NUM_BANDS> {
    fn drop(&mut self) {
        persistence::save_to_json_file(
            &self.settings.ui.show_options,
            &self
                .settings
                .persistence_dir
                .join("show_options.json")
                .as_path(),
        );
        persistence::save_to_json_file(
            &self.eqs,
            &self.settings.persistence_dir.join("eqs.json").as_path(),
        );
    }
}
