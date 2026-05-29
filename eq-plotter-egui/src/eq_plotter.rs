use crate::*;
use app_lib::persistence;

pub type AppSettings<const NUM_BANDS: usize> = app_lib::settings::Settings<f64, NUM_BANDS>;
pub type UiParams<const NUM_BANDS: usize> = egui_lib::Params<f64, NUM_BANDS>;
pub type UiSettings = egui_lib::Settings<f64>;

pub struct EqPlotter<const NUM_BANDS: usize> {
    params: UiParams<NUM_BANDS>,
    ui_settings: UiSettings,
    params_file_path: std::path::PathBuf,
}

impl<const NUM_BANDS: usize> eframe::App for EqPlotter<NUM_BANDS> {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .inner_margin(20)
                    .fill(self.ui_settings.color_palette.background),
            )
            .show_inside(ui, |ui| {
                egui_lib::draw::<_, _, 0, 0>(ui, &mut self.params, &self.ui_settings, &None);
            });
    }
}

impl<const NUM_BANDS: usize> EqPlotter<NUM_BANDS> {
    pub fn new(app_settings: AppSettings<NUM_BANDS>, color_palette: colors::ColorPalette) -> Self {
        let params_file_path = app_settings.persistence_dir.join("params.json");
        let params = if let Some(params) =
            persistence::create_from_json_file::<UiParams<NUM_BANDS>>(&&params_file_path.as_path())
        {
            params
        } else {
            UiParams::<NUM_BANDS> {
                show_options: app_lib::settings::ui::ShowOptions::new_all_enabled(),
                eqs: app_settings.init_eqs.clone(),
                sample_rate: app_settings.init_sample_rate,
                drag_eq_index: usize::MAX,
            }
        };
        Self {
            params: params,
            ui_settings: UiSettings {
                app: app_settings.ui,
                color_palette: color_palette,
            },
            params_file_path: params_file_path,
        }
    }
}

impl<const NUM_BANDS: usize> Drop for EqPlotter<NUM_BANDS> {
    fn drop(&mut self) {
        persistence::save_to_json_file(&self.params, &self.params_file_path.as_path());
    }
}
