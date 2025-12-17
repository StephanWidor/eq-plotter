use audio_lib::biquad;
use audio_lib::eq;
use audio_lib::utils;
use num::complex::ComplexFloat;
use num_traits::Float;
use num_traits::Pow;
use num_traits::cast::FromPrimitive;

pub struct EqPlotter {
    sample_rate: f64,
    eq: eq::Eq<f64>,
}

impl Default for EqPlotter {
    fn default() -> Self {
        Self {
            sample_rate: 48000.0,
            eq: Self::DEFAULT_EQ,
        }
    }
}

impl EqPlotter {
    pub const MIN_GAIN_DB: f64 = -20.0;
    pub const MAX_GAIN_DB: f64 = 20.0;
    pub const MIN_FREQUENCY: f64 = 10.0;
    pub const MIN_LOG_FREQUENCY: f64 = 1.0; // 10.0.log10();
    pub const MAX_FREQUENCY: f64 = 20000.0;
    pub const MAX_LOG_FREQUENCY: f64 = 4.3010299956639813; // 20000.0.log10();
    pub const MIN_Q: f64 = 0.1;
    pub const MAX_Q: f64 = 10.0;
    pub const DEFAULT_EQ: eq::Eq<f64> = eq::Eq {
        gain: eq::Gain::Db(-3.0),
        frequency: eq::Frequency::Hz(1000.0),
        q: 0.7,
        eq_type: eq::EqType::Peak,
    };

    pub fn log_frequency_to_string<F: Float + FromPrimitive + std::fmt::Display>(
        log_frequency: F,
    ) -> String {
        //format!("{} Hz", utils::log_to_frequency(log_frequency.round()))
        format!("{}", utils::log_to_frequency(log_frequency).round())
    }

    pub fn string_to_log_frequency<F: Float + FromPrimitive + std::str::FromStr>(
        frequency_string: &str,
    ) -> Option<F> {
        let trimmed_string = frequency_string.trim_end_matches(&[' ', 'H', 'z']);
        trimmed_string
            .parse::<F>()
            .ok()
            .map(utils::frequency_to_log)
        //            .unwrap_or(F::from(EqPlotter::MIN_LOG_FREQUENCY).unwrap())
    }

    pub fn draw(ui: &mut egui::Ui, eq_in: &eq::Eq<f64>, sample_rate: f64) -> eq::Eq<f64> {
        let mut eq = eq_in.clone();
        let mut gain_db = eq.gain.db();
        let mut log_frequency = eq.frequency.log_hz();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                egui::ComboBox::from_label("")
                    .selected_text(eq.eq_type.to_string())
                    .width(220.0)
                    .show_ui(ui, |ui| {
                        for eq_type in eq::EqType::ALL.iter() {
                            ui.selectable_value(&mut eq.eq_type, *eq_type, eq_type.to_string());
                        }
                    });

                if eq.eq_type.has_frequency() {
                    ui.add(
                        egui::Slider::new(
                            &mut log_frequency,
                            EqPlotter::MIN_LOG_FREQUENCY..=EqPlotter::MAX_LOG_FREQUENCY,
                        )
                        .custom_formatter(|log_frequency, _| {
                            Self::log_frequency_to_string(log_frequency)
                        })
                        .custom_parser(Self::string_to_log_frequency)
                        .prefix("frequency: ")
                        .suffix("Hz"),
                    );
                    eq.frequency = eq::Frequency::LogHz(log_frequency);
                }

                if eq.eq_type.has_gain_db() {
                    ui.add(
                        egui::Slider::new(
                            &mut gain_db,
                            EqPlotter::MIN_GAIN_DB..=EqPlotter::MAX_GAIN_DB,
                        )
                        .prefix("gain: ")
                        .suffix("dB"),
                    );
                    eq.gain = eq::Gain::Db(gain_db);
                }

                if eq.eq_type.has_q() {
                    ui.add(
                        egui::Slider::new(&mut eq.q, EqPlotter::MIN_Q..=EqPlotter::MAX_Q)
                            .prefix("Q: "),
                    );
                }
            });

            let log_frequency_formatter =
                |mark: egui_plot::GridMark, _range: &std::ops::RangeInclusive<f64>| -> String {
                    let log_frequency = mark.value;
                    if log_frequency.fract().abs() < 1e-6 {
                        let frequency = utils::log_to_frequency(log_frequency);
                        format!("{}", frequency)
                    } else {
                        String::new()
                    }
                };

            let coefficients = biquad::coefficients::Coefficients::from_eq(&eq, sample_rate);
            let frequency_response =
                biquad::utils::make_frequency_response_function(&coefficients, sample_rate);
            let plot_size = 400.0;

            ui.vertical(|ui| {
                egui_plot::Plot::new("Gain (dB)")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .width(plot_size)
                    .height(plot_size)
                    .auto_bounds([false, false])
                    .custom_x_axes(vec![
                        egui_plot::AxisHints::new_x().formatter(log_frequency_formatter),
                    ])
                    .label_formatter(|_, point| {
                        format!("{} Hz, {:.2} dB", 10.0.pow(point.x) as i32, point.y)
                    })
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                            [EqPlotter::MIN_LOG_FREQUENCY, EqPlotter::MIN_GAIN_DB],
                            [EqPlotter::MAX_LOG_FREQUENCY, EqPlotter::MAX_GAIN_DB],
                        ));
                        let gain_points = egui_plot::PlotPoints::from_explicit_callback(
                            |log_frequency| {
                                utils::amplitude_to_db(
                                    frequency_response(10.0.pow(log_frequency)).abs(),
                                )
                            },
                            EqPlotter::MIN_LOG_FREQUENCY..=EqPlotter::MAX_LOG_FREQUENCY,
                            1000,
                        );
                        plot_ui.line(egui_plot::Line::new(gain_points).name("Gain Response"));
                    });

                egui_plot::Plot::new("Phase")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .width(plot_size)
                    .height(plot_size)
                    .auto_bounds([false, false])
                    .custom_x_axes(vec![
                        egui_plot::AxisHints::new_x().formatter(log_frequency_formatter),
                    ])
                    .label_formatter(|_, point| {
                        format!("{} Hz, {:.2} rad", 10.0.pow(point.x) as i32, point.y)
                    })
                    .show_x(false)
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                            [EqPlotter::MIN_LOG_FREQUENCY, -std::f64::consts::PI],
                            [EqPlotter::MAX_LOG_FREQUENCY, std::f64::consts::PI],
                        ));
                        let phase_points = egui_plot::PlotPoints::from_explicit_callback(
                            |log_frequency| frequency_response(10.0.pow(log_frequency)).arg(),
                            EqPlotter::MIN_LOG_FREQUENCY..=EqPlotter::MAX_LOG_FREQUENCY,
                            1000,
                        );
                        plot_ui.line(egui_plot::Line::new(phase_points).name("Phase Response"));
                    });
            });

            ui.vertical(|ui| {
                egui_plot::Plot::new("Impulse Response")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .width(plot_size)
                    .height(plot_size)
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        let impulse_response =
                            biquad::utils::impulse_response(&coefficients, 0.001, 10, 1024);
                        let response_points = egui_plot::PlotPoints::from_ys_f64(&impulse_response);
                        plot_ui
                            .line(egui_plot::Line::new(response_points).name("Impulse Response"));
                    });

                egui_plot::Plot::new("Poles And Zeros")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .width(plot_size)
                    .height(plot_size)
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                            [-1.5, -1.5],
                            [1.5, 1.5],
                        ));
                        let unit_circle_points = egui_plot::PlotPoints::from_parametric_callback(
                            |angle| (angle.cos(), angle.sin()),
                            0.0..=2.0 * std::f64::consts::PI,
                            100,
                        );
                        plot_ui.line(egui_plot::Line::new(unit_circle_points).name("Unit Circle"));

                        let poles = biquad::utils::poles(&coefficients)
                            .iter()
                            .map(|pole| [pole.re, pole.im])
                            .collect::<Vec<_>>();
                        let pole_markers = egui_plot::Points::new(poles)
                            .name("Poles")
                            .filled(true)
                            .radius(3.0);
                        plot_ui.points(pole_markers);

                        let zeros = biquad::utils::zeros(&coefficients)
                            .iter()
                            .map(|zero| [zero.re, zero.im])
                            .collect::<Vec<_>>();
                        let zero_markers = egui_plot::Points::new(zeros)
                            .name("Zeros")
                            .filled(true)
                            .radius(3.0);
                        plot_ui.points(zero_markers);

                        if !biquad::utils::is_stable(&coefficients) {
                            plot_ui.text(
                                egui_plot::Text::new(
                                    egui_plot::PlotPoint::new(0.0, 0.5),
                                    "Biquad is not stable!",
                                )
                                .color(egui::Color32::RED),
                            );
                        }
                    });
            });
        });
        eq
    }
}

impl eframe::App for EqPlotter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .inner_margin(20)
                    .fill(egui::Color32::from_rgb(32, 35, 38)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    self.eq = EqPlotter::draw(ui, &self.eq, self.sample_rate);
                });
            });
    }
}
