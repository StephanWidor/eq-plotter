mod biquad;
mod eq;
mod utils;

use num::complex::ComplexFloat;
use num_traits::Pow;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
fn main() -> eframe::Result {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("no global window exists");
        let document = window.document().expect("should have a document on window");
        let canvas = document
            .get_element_by_id("canvas_id")
            .expect("should have a canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("should be a canvas");

        let web_options = eframe::WebOptions::default();
        let web_runner = eframe::WebRunner::new();
        web_runner
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::<EqPlotter>::default())),
            )
            .await
            .expect("failed to start WebRunner");
    });
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "EQ Plotter",
        options,
        Box::new(|_cc| Ok(Box::<EqPlotter>::default())),
    )
}

struct EqPlotter {
    sample_rate: f64,
    log_frequency: f64,
    gain_db: f64,
    q: f64,
    eq: eq::EQ<f64>,
}

impl Default for EqPlotter {
    fn default() -> Self {
        let log_frequency = 1000.0.log10();
        let gain_db = -3.0;
        let q = 0.7;
        Self {
            sample_rate: 48000.0,
            log_frequency: log_frequency,
            gain_db: gain_db,
            q: q,
            eq: eq::EQ::Peak(eq::Peak {
                frequency: log_frequency.pow(10.0),
                gain_db: gain_db,
                q: q,
            }),
        }
    }
}

impl EqPlotter {
    const MIN_GAIN_DB: f64 = -20.0;
    const MAX_GAIN_DB: f64 = 20.0;
    const MIN_LOG_FREQUENCY: f64 = 1.0; // 10.0.log10();
    const MAX_LOG_FREQUENCY: f64 = 4.3010299956639813; // 20000.0.log10();
    const MIN_Q: f64 = 0.1;
    const MAX_Q: f64 = 10.0;
}

impl eframe::App for EqPlotter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            sample_rate,
            log_frequency,
            gain_db,
            q,
            eq,
        } = self;

        let mut frequency = 10.0.pow(*log_frequency);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .inner_margin(20)
                    .fill(egui::Color32::from_rgb(32, 35, 38)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        egui::ComboBox::from_label("")
                            .selected_text(eq.to_string())
                            .width(220.0)
                            .show_ui(ui, |ui| {
                                for eq_type in eq::EQ::all(frequency, *gain_db, *q) {
                                    ui.selectable_value(eq, eq_type, eq_type.to_string());
                                }
                            });

                        ui.add_enabled(
                            eq.has_frequency(),
                            egui::Slider::new(
                                &mut *log_frequency,
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

                        ui.add_enabled(
                            eq.has_gain_db(),
                            egui::Slider::new(
                                &mut *gain_db,
                                EqPlotter::MIN_GAIN_DB..=EqPlotter::MAX_GAIN_DB,
                            )
                            .prefix("gain: ")
                            .suffix("dB"),
                        );

                        ui.add_enabled(
                            eq.has_q(),
                            egui::Slider::new(&mut *q, EqPlotter::MIN_Q..=EqPlotter::MAX_Q)
                                .prefix("Q: "),
                        );

                        frequency = 10.0.pow(*log_frequency);
                        eq.set_parameters(frequency, *gain_db, *q);
                    });

                    let log_frequency_formatter = |mark: egui_plot::GridMark,
                                                   _range: &std::ops::RangeInclusive<f64>|
                     -> String {
                        let log_frequency = mark.value;
                        if log_frequency.fract().abs() < 1e-6 {
                            let frequency = 10.0.pow(mark.value);
                            format!("{}", frequency)
                        } else {
                            String::new()
                        }
                    };

                    let coefficients =
                        biquad::coefficients::Coefficients::from_eq(eq, *sample_rate);
                    let frequency_response = biquad::utils::make_frequency_response_function(
                        &coefficients,
                        *sample_rate,
                    );
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
                                plot_ui.line(egui_plot::Line::new("Gain Response", gain_points));
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
                                    |log_frequency| {
                                        frequency_response(10.0.pow(log_frequency)).arg()
                                    },
                                    EqPlotter::MIN_LOG_FREQUENCY..=EqPlotter::MAX_LOG_FREQUENCY,
                                    1000,
                                );
                                plot_ui.line(egui_plot::Line::new("Phase Response", phase_points));
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
                                let response_points =
                                    egui_plot::PlotPoints::from_ys_f64(&impulse_response);
                                plot_ui.line(egui_plot::Line::new(
                                    "Impulse Response",
                                    response_points,
                                ));
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
                                let unit_circle_points =
                                    egui_plot::PlotPoints::from_parametric_callback(
                                        |angle| (angle.cos(), angle.sin()),
                                        0.0..=2.0 * std::f64::consts::PI,
                                        100,
                                    );
                                plot_ui
                                    .line(egui_plot::Line::new("Unit Circle", unit_circle_points));

                                let poles = biquad::utils::poles(&coefficients)
                                    .iter()
                                    .map(|pole| [pole.re, pole.im])
                                    .collect::<Vec<_>>();
                                let pole_markers = egui_plot::Points::new("Poles", poles)
                                    .filled(true)
                                    .radius(3.0);
                                plot_ui.points(pole_markers);

                                let zeros = biquad::utils::zeros(&coefficients)
                                    .iter()
                                    .map(|zero| [zero.re, zero.im])
                                    .collect::<Vec<_>>();
                                let zero_markers = egui_plot::Points::new("Zeros", zeros)
                                    .filled(true)
                                    .radius(3.0);
                                plot_ui.points(zero_markers);

                                if !biquad::utils::is_stable(&coefficients) {
                                    plot_ui.text(
                                        egui_plot::Text::new(
                                            "Stability Warning",
                                            egui_plot::PlotPoint::new(0.0, 0.5),
                                            "Biquad is not stable!",
                                        )
                                        .color(egui::Color32::RED),
                                    );
                                }
                            });
                    });
                });
            });
    }
}
