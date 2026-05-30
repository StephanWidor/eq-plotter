use crate::*;
use app_lib::persistence;

pub type AppSettings<const NUM_BANDS: usize> = app_lib::settings::Settings<f64, NUM_BANDS>;
pub type UiParams<const NUM_BANDS: usize> = egui_lib::Params<f64, NUM_BANDS>;
pub type UiSettings = egui_lib::Settings<f64>;
pub type Presets<const NUM_BANDS: usize> = app_lib::presets::Presets<f64, NUM_BANDS>;

pub struct EqPlotter<const NUM_BANDS: usize> {
    params: UiParams<NUM_BANDS>,
    presets: Presets<NUM_BANDS>,
    ui_settings: UiSettings,
    persistence_dir: std::path::PathBuf,
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
                egui_lib::draw::<_, _, 0, 0>(
                    ui,
                    &mut self.params,
                    &mut self.presets,
                    &self.ui_settings,
                    &None,
                );
            });
    }
}

impl<const NUM_BANDS: usize> EqPlotter<NUM_BANDS> {
    pub fn new(app_settings: AppSettings<NUM_BANDS>, color_palette: colors::ColorPalette) -> Self {
        let params = if let Some(params) = persistence::create_from_json_file::<UiParams<NUM_BANDS>>(
            &Self::params_file_path(&app_settings.persistence_dir).as_path(),
        ) {
            params
        } else {
            UiParams::<NUM_BANDS> {
                show_options: app_lib::settings::ui::ShowOptions::new_all_enabled(),
                eqs: app_settings.init_eqs.clone(),
                sample_rate: app_settings.init_sample_rate,
                drag_eq_index: usize::MAX,
                preset_selection: app_lib::presets::Selection::None,
            }
        };
        let presets = if let Some(presets) = persistence::create_from_json_file::<Presets<NUM_BANDS>>(
            &Self::presets_file_path(&app_settings.persistence_dir).as_path(),
        ) {
            presets
        } else {
            Presets::<NUM_BANDS>::new()
        };
        Self {
            params: params,
            presets: presets,
            ui_settings: UiSettings {
                app: app_settings.ui,
                color_palette: color_palette,
            },
            persistence_dir: app_settings.persistence_dir,
        }
    }

    fn params_file_path(persistence_dir: &std::path::PathBuf) -> std::path::PathBuf {
        persistence_dir.join("params.json")
    }

    fn presets_file_path(presets_dir: &std::path::PathBuf) -> std::path::PathBuf {
        presets_dir.join("presets.json")
    }
}

impl<const NUM_BANDS: usize> Drop for EqPlotter<NUM_BANDS> {
    fn drop(&mut self) {
        persistence::save_to_json_file(
            &self.params,
            &Self::params_file_path(&self.persistence_dir),
        );
        persistence::save_to_json_file(
            &self.presets,
            &Self::presets_file_path(&self.persistence_dir),
        );
    }
}
