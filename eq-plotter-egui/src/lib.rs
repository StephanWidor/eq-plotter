use audio_lib::biquad;
use audio_lib::eq;
use audio_lib::utils;
use num::complex::ComplexFloat;
use num_traits::Pow;

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
        gain_db: -3.0,
        frequency: 1000.0,
        q: 0.7,
        eq_type: eq::EqType::Peak,
    };

    pub fn draw(ui: &mut egui::Ui, eq: &mut eq::Eq<f64>, sample_rate: f64) {
        let mut log_frequency = eq.frequency.log10();
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

                ui.add_enabled(
                    eq.eq_type.has_frequency(),
                    egui::Slider::new(
                        &mut log_frequency,
                        EqPlotter::MIN_LOG_FREQUENCY..=EqPlotter::MAX_LOG_FREQUENCY,
                    )
                    .custom_formatter(|log_frequency, _| {
                        10.0.pow(log_frequency).round().to_string()
                    })
                    .custom_parser(|as_string| {
                        let parsed = String::from(as_string).parse::<f64>();
                        match parsed {
                            Ok(frequency) => Some(frequency.log10()),
                            Err(_) => None,
                        }
                    })
                    .prefix("freqency: ")
                    .suffix("Hz"),
                );
                eq.frequency = 10.0.pow(log_frequency);

                ui.add_enabled(
                    eq.eq_type.has_gain_db(),
                    egui::Slider::new(
                        &mut eq.gain_db,
                        EqPlotter::MIN_GAIN_DB..=EqPlotter::MAX_GAIN_DB,
                    )
                    .prefix("gain: ")
                    .suffix("dB"),
                );

                ui.add_enabled(
                    eq.eq_type.has_q(),
                    egui::Slider::new(&mut eq.q, EqPlotter::MIN_Q..=EqPlotter::MAX_Q).prefix("Q: "),
                );
            });

            let log_frequency_formatter =
                |mark: egui_plot::GridMark, _range: &std::ops::RangeInclusive<f64>| -> String {
                    let log_frequency = mark.value;
                    if log_frequency.fract().abs() < 1e-6 {
                        let frequency = 10.0.pow(mark.value);
                        format!("{}", frequency)
                    } else {
                        String::new()
                    }
                };

            let coefficients = biquad::coefficients::Coefficients::from_eq(eq, sample_rate);
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
    }
}

impl eframe::App for EqPlotter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { sample_rate, eq } = self;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .inner_margin(20)
                    .fill(egui::Color32::from_rgb(32, 35, 38)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    EqPlotter::draw(ui, eq, *sample_rate);
                });
            });
    }
}
