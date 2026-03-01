use crate::*;
use app_lib as app;
use audio_lib::eq;
pub use plotter::SpectrumData;

pub fn draw<const NUM_SPECTRUM_BINS: usize, const NUM_SPECTRUM_CHANNELS: usize>(
    ui: &mut egui::Ui,
    eqs: &mut [eq::Eq<f64>],
    selected_eq_index: &mut usize,
    spectrum_data: &Option<SpectrumData<NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>>,
    show_options: &mut options::ShowOptions,
    sample_rate: f64,
) {
    let ui_size = ui.available_size();

    ui.horizontal(|ui| {
        let control_width = 250_f32;
        control::add_eq_controls(
            ui,
            egui::Vec2::new(control_width, ui_size.y),
            eqs,
            show_options,
        );

        if !(show_options.gain
            || show_options.phase
            || show_options.impulse_response
            || show_options.poles_and_zeros)
        {
            return;
        }
        let available_size = egui::Vec2::new(0.96_f32 * (ui_size.x - control_width), ui_size.y);
        let plot_spectrum_data = if show_options.signal_gain_spectrum {
            spectrum_data
        } else {
            &None
        };
        plotter::add_plots::<NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>(
            ui,
            &available_size,
            eqs,
            selected_eq_index,
            plot_spectrum_data,
            show_options,
            sample_rate,
        );
    });
}

pub struct EqPlotter {
    eqs: Vec<eq::Eq<f64>>,
    selected_eq_index: usize,
    sample_rate: f64,
    show_options: options::ShowOptions,
}

impl eframe::App for EqPlotter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .inner_margin(20)
                    .fill(constants::BACKGROUND_COLOR),
            )
            .show(ctx, |ui| {
                draw::<1, 1>(
                    ui,
                    &mut self.eqs,
                    &mut self.selected_eq_index,
                    &None,
                    &mut self.show_options,
                    self.sample_rate,
                );
            });
    }
}

impl EqPlotter {
    pub fn new(num_bands: usize) -> Self {
        assert!(num_bands > 0);
        let mut eq_plotter = Self {
            eqs: vec![constants::INIT_EQ; num_bands],
            selected_eq_index: usize::MAX,
            sample_rate: 48000.0,
            show_options: options::ShowOptions::new_all_enabled(),
        };
        eq_plotter.eqs[0] = app::DEFAULT_EQ;
        eq_plotter
    }
}
