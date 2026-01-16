use app_lib as app;
use audio_lib::biquad;
use audio_lib::eq;
use audio_lib::utils;
use num::complex::ComplexFloat;
use num_traits::Pow;

pub struct EqPlotter {
    sample_rate: f64,
    eqs: Vec<eq::Eq<f64>>,
}

impl eframe::App for EqPlotter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .inner_margin(20)
                    .fill(egui::Color32::from_rgb(
                        app::UI_BACKGROUND_COLOR[0],
                        app::UI_BACKGROUND_COLOR[1],
                        app::UI_BACKGROUND_COLOR[2],
                    )),
            )
            .show(ctx, |ui| {
                EqPlotter::draw(ui, &mut self.eqs, self.sample_rate);
            });
    }
}

impl EqPlotter {
    pub const WINDOW_SIZE: [u32; 2] = [1200, 900]; // [width, height]
    const EQ_COLORS: [egui::Color32; 5] = [
        egui::Color32::ORANGE,
        egui::Color32::GREEN,
        egui::Color32::RED,
        egui::Color32::YELLOW,
        egui::Color32::BLUE,
    ];
    const MULTI_BAND_COLOR: egui::Color32 = egui::Color32::LIGHT_GRAY;
    pub const START_EQ: eq::Eq<f64> = eq::Eq {
        gain: eq::Gain::Db(0.0),
        frequency: eq::Frequency::Hz(1000.0),
        q: 0.7,
        eq_type: eq::EqType::Bypassed,
    };

    pub fn new(num_bands: usize) -> Self {
        let mut eq_plotter = Self {
            sample_rate: 48000.0,
            eqs: vec![Self::START_EQ; num_bands],
        };
        if num_bands > 0 {
            eq_plotter.eqs[0] = app::DEFAULT_EQ;
        }
        eq_plotter
    }

    pub fn draw(ui: &mut egui::Ui, eqs: &mut [eq::Eq<f64>], sample_rate: f64) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for (index, eq) in eqs.iter_mut().enumerate() {
                    Self::eq_control(ui, Self::EQ_COLORS[index % Self::EQ_COLORS.len()], eq);
                }
            });

            let active_eqs = eqs
                .iter()
                .map(|eq| eq.eq_type.is_active())
                .collect::<Vec<_>>();
            let coefficients = eqs
                .iter()
                .map(|eq| biquad::coefficients::Coefficients::from_eq(&eq, sample_rate))
                .collect::<Vec<_>>();
            let frequency_responses = coefficients
                .iter()
                .map(|c| biquad::utils::make_frequency_response(&c, sample_rate))
                .collect::<Vec<_>>();
            let multiband_frequency_response =
                biquad::utils::multiband::make_frequency_response(&coefficients, sample_rate);

            let eps_for_impulse = 0.001;
            let hold_for_impulse = 10;
            let max_length_for_impulse = 1024;
            let impulse_responses = coefficients
                .iter()
                .map(|c| {
                    biquad::utils::impulse_response_for_coefficients(
                        &c,
                        eps_for_impulse,
                        hold_for_impulse,
                        max_length_for_impulse,
                    )
                })
                .collect::<Vec<_>>();
            let multiband_impulse_response =
                biquad::utils::multiband::impulse_response_for_coefficients(
                    &coefficients,
                    eps_for_impulse,
                    hold_for_impulse,
                    max_length_for_impulse,
                );

            let plot_size = 400.0;

            ui.vertical(|ui| {
                Self::gain_plot(
                    ui,
                    &frequency_responses,
                    &active_eqs,
                    &multiband_frequency_response,
                    plot_size,
                );
                Self::phase_plot(
                    ui,
                    &frequency_responses,
                    &active_eqs,
                    &multiband_frequency_response,
                    plot_size,
                );
            });

            ui.vertical(|ui| {
                Self::impulse_response_plot(
                    ui,
                    &impulse_responses,
                    &active_eqs,
                    &multiband_impulse_response,
                    plot_size,
                );
                Self::poles_and_zeros_plot(ui, &coefficients, &active_eqs, plot_size);
            });
            eqs
        });
    }

    fn eq_control(ui: &mut egui::Ui, color: egui::Color32, eq: &mut eq::Eq<f64>) {
        let mut gain_db = eq.gain.db();
        let mut log_frequency = eq.frequency.log_hz();
        egui::Frame::group(ui.style())
            .fill(color)
            .multiply_with_opacity(0.2_f32)
            .corner_radius(5)
            .outer_margin(20)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    egui::ComboBox::from_id_salt(ui.next_auto_id())
                        .selected_text(eq.eq_type.to_string())
                        .width(250.0)
                        .show_ui(ui, |ui| {
                            for eq_type in eq::EqType::ALL.iter() {
                                ui.selectable_value(&mut eq.eq_type, *eq_type, eq_type.to_string());
                            }
                        });

                    if eq.eq_type.has_frequency() {
                        ui.add(
                            egui::Slider::new(
                                &mut log_frequency,
                                app::MIN_LOG_FREQUENCY..=app::MAX_LOG_FREQUENCY,
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
                            egui::Slider::new(&mut gain_db, app::MIN_GAIN_DB..=app::MAX_GAIN_DB)
                                .prefix("gain: ")
                                .suffix("dB"),
                        );
                        eq.gain = eq::Gain::Db(gain_db);
                    }

                    if eq.eq_type.has_q() {
                        ui.add(egui::Slider::new(&mut eq.q, app::MIN_Q..=app::MAX_Q).prefix("Q: "));
                    }
                });
            });
    }

    fn gain_plot(
        ui: &mut egui::Ui,
        frequency_responses: &[impl Fn(f64) -> num::Complex<f64>],
        active_eqs: &[bool],
        multiband_frequency_response: &impl Fn(f64) -> num::Complex<f64>,
        plot_size: f32,
    ) {
        egui_plot::Plot::new("Gain (dB)")
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .width(plot_size)
            .height(plot_size)
            .auto_bounds([false, false])
            .custom_x_axes(vec![
                egui_plot::AxisHints::new_x()
                    .label("Frequency (Hz)")
                    .formatter(|_, _| String::new())
                    .placement(egui_plot::VPlacement::Top),
                egui_plot::AxisHints::new_x().formatter(Self::log_frequency_formatter),
            ])
            .custom_y_axes(vec![
                egui_plot::AxisHints::new_y()
                    .label("Gain (dB)")
                    .formatter(|_, _| String::new()),
                egui_plot::AxisHints::new_y().placement(egui_plot::HPlacement::Right),
            ])
            .label_formatter(|_, point| {
                format!(
                    "{} Hz, {:.2} dB",
                    utils::log_to_frequency(point.x) as i32,
                    point.y
                )
            })
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                    [app::MIN_LOG_FREQUENCY, app::MIN_GAIN_DB],
                    [app::MAX_LOG_FREQUENCY, app::MAX_GAIN_DB],
                ));

                let mut num_active = 0;
                for ((index, response), active) in
                    frequency_responses.iter().enumerate().zip(active_eqs)
                {
                    if !*active {
                        continue;
                    }
                    num_active += 1;
                    // could be optimized: we don't need to calculate frequencies from log for every response
                    let gain_points = egui_plot::PlotPoints::from_explicit_callback(
                        |log_frequency| {
                            utils::amplitude_to_db(
                                response(utils::log_to_frequency(log_frequency)).abs(),
                            )
                        },
                        app::MIN_LOG_FREQUENCY..=app::MAX_LOG_FREQUENCY,
                        1000,
                    );
                    plot_ui.line(
                        egui_plot::Line::new(gain_points)
                            .color(Self::EQ_COLORS[index % Self::EQ_COLORS.len()]),
                    );
                }
                if num_active > 1 {
                    let gain_points = egui_plot::PlotPoints::from_explicit_callback(
                        |log_frequency| {
                            utils::amplitude_to_db(
                                multiband_frequency_response(utils::log_to_frequency(
                                    log_frequency,
                                ))
                                .abs(),
                            )
                        },
                        app::MIN_LOG_FREQUENCY..=app::MAX_LOG_FREQUENCY,
                        1000,
                    );
                    plot_ui.line(
                        egui_plot::Line::new(gain_points)
                            .color(Self::MULTI_BAND_COLOR)
                            .name("multiband"),
                    );
                }
            });
    }

    fn phase_plot(
        ui: &mut egui::Ui,
        frequency_responses: &[impl Fn(f64) -> num::Complex<f64>],
        active_eqs: &[bool],
        multiband_frequency_response: &impl Fn(f64) -> num::Complex<f64>,
        plot_size: f32,
    ) {
        egui_plot::Plot::new("Phase")
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .width(plot_size)
            .height(plot_size)
            .auto_bounds([false, false])
            .custom_x_axes(vec![
                egui_plot::AxisHints::new_x()
                    .formatter(Self::log_frequency_formatter)
                    .placement(egui_plot::VPlacement::Top),
                egui_plot::AxisHints::new_x()
                    .label("Frequency (Hz)")
                    .formatter(|_, _| String::new()),
            ])
            .custom_y_axes(vec![
                egui_plot::AxisHints::new_y()
                    .label("Phase (rad)")
                    .formatter(|_, _| String::new()),
                egui_plot::AxisHints::new_y().placement(egui_plot::HPlacement::Right),
            ])
            .label_formatter(|_, point| {
                format!("{} Hz, {:.2} rad", 10.0.pow(point.x) as i32, point.y)
            })
            .show_x(false)
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                    [app::MIN_LOG_FREQUENCY, -std::f64::consts::PI],
                    [app::MAX_LOG_FREQUENCY, std::f64::consts::PI],
                ));

                let mut num_active = 0;
                for ((index, response), active) in
                    frequency_responses.iter().enumerate().zip(active_eqs)
                {
                    if !*active {
                        continue;
                    }
                    num_active += 1;

                    // could be optimized: we don't need to calculate frequencies from log for every response
                    let phase_points = egui_plot::PlotPoints::from_explicit_callback(
                        |log_frequency| response(utils::log_to_frequency(log_frequency)).arg(),
                        app::MIN_LOG_FREQUENCY..=app::MAX_LOG_FREQUENCY,
                        1000,
                    );
                    plot_ui.line(
                        egui_plot::Line::new(phase_points)
                            .color(Self::EQ_COLORS[index % Self::EQ_COLORS.len()]),
                    );
                }

                if num_active > 1 {
                    let phase_points = egui_plot::PlotPoints::from_explicit_callback(
                        |log_frequency| {
                            multiband_frequency_response(utils::log_to_frequency(log_frequency))
                                .arg()
                        },
                        app::MIN_LOG_FREQUENCY..=app::MAX_LOG_FREQUENCY,
                        1000,
                    );
                    plot_ui.line(
                        egui_plot::Line::new(phase_points)
                            .color(Self::MULTI_BAND_COLOR)
                            .name("multiband"),
                    );
                }
            });
    }

    fn impulse_response_plot(
        ui: &mut egui::Ui,
        impulse_responses: &[Vec<f64>],
        active_eqs: &[bool],
        multiband_impulse_responses: &Vec<f64>,
        plot_size: f32,
    ) {
        egui_plot::Plot::new("Impulse Response")
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .width(plot_size)
            .height(plot_size)
            .custom_x_axes(vec![
                egui_plot::AxisHints::new_x()
                    .label("Samples")
                    .formatter(|_, _| String::new())
                    .placement(egui_plot::VPlacement::Top),
                egui_plot::AxisHints::new_x(),
            ])
            .custom_y_axes(vec![
                egui_plot::AxisHints::new_y(),
                egui_plot::AxisHints::new_y()
                    .label("Impulse Response (Amplitude)")
                    .formatter(|_, _| String::new())
                    .placement(egui_plot::HPlacement::Right),
            ])
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                let mut num_active = 0;
                for ((index, response), active) in
                    impulse_responses.iter().enumerate().zip(active_eqs)
                {
                    if !*active {
                        continue;
                    }
                    num_active += 1;
                    plot_ui.line(
                        egui_plot::Line::new(egui_plot::PlotPoints::from_ys_f64(&response))
                            .color(Self::EQ_COLORS[index % Self::EQ_COLORS.len()]),
                    );
                }
                if num_active > 1 {
                    plot_ui.line(
                        egui_plot::Line::new(egui_plot::PlotPoints::from_ys_f64(
                            &multiband_impulse_responses,
                        ))
                        .name("Impulse Response")
                        .color(Self::MULTI_BAND_COLOR)
                        .name("multiband"),
                    );
                }
            });
    }

    fn poles_and_zeros_plot(
        ui: &mut egui::Ui,
        coefficients: &[biquad::coefficients::Coefficients<f64>],
        active_eqs: &[bool],
        plot_size: f32,
    ) {
        egui_plot::Plot::new("Poles And Zeros")
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .width(plot_size)
            .height(plot_size)
            .custom_x_axes(vec![
                egui_plot::AxisHints::new_x().placement(egui_plot::VPlacement::Top),
                egui_plot::AxisHints::new_x()
                    .label("Re")
                    .formatter(|_, _| String::new()),
            ])
            .custom_y_axes(vec![
                egui_plot::AxisHints::new_y(),
                egui_plot::AxisHints::new_y()
                    .label("Im")
                    .formatter(|_, _| String::new())
                    .placement(egui_plot::HPlacement::Right),
            ])
            .legend(egui_plot::Legend::default())
            .show(ui, |plot_ui| {
                let mut unstable_biquad = false;
                plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                    [-1.5, -1.5],
                    [1.5, 1.5],
                ));

                let unit_circle = egui_plot::PlotPoints::from_parametric_callback(
                    |angle| (angle.cos(), angle.sin()),
                    0.0..=2.0 * std::f64::consts::PI,
                    100,
                );
                plot_ui.line(
                    egui_plot::Line::new(unit_circle)
                        .width(1_f32)
                        .color(egui::Color32::GRAY),
                );

                for ((index, c), active) in coefficients.iter().enumerate().zip(active_eqs) {
                    if !*active {
                        continue;
                    }
                    let poles = biquad::utils::poles(&c)
                        .iter()
                        .map(|pole| [pole.re, pole.im])
                        .collect::<Vec<_>>();
                    plot_ui.points(
                        egui_plot::Points::new(poles)
                            .name("Poles")
                            .color(Self::EQ_COLORS[index % Self::EQ_COLORS.len()])
                            .shape(egui_plot::MarkerShape::Cross)
                            .radius(4.0),
                    );

                    let zeros = biquad::utils::zeros(&c)
                        .iter()
                        .map(|zero| [zero.re, zero.im])
                        .collect::<Vec<_>>();
                    plot_ui.points(
                        egui_plot::Points::new(zeros)
                            .name("Zeros")
                            .color(Self::EQ_COLORS[index % Self::EQ_COLORS.len()])
                            .shape(egui_plot::MarkerShape::Circle)
                            .filled(false)
                            .radius(2.0),
                    );
                    if !biquad::utils::is_stable(c) {
                        unstable_biquad = true;
                    }
                }

                if unstable_biquad {
                    plot_ui.text(
                        egui_plot::Text::new(
                            egui_plot::PlotPoint::new(0.0, 0.5),
                            "Biquad is not stable!",
                        )
                        .color(egui::Color32::RED),
                    );
                }
            });
    }

    pub fn log_frequency_to_string<F: utils::Float + std::fmt::Display>(
        log_frequency: F,
    ) -> String {
        format!("{}", utils::log_to_frequency(log_frequency).round())
    }

    pub fn string_to_log_frequency<F: utils::Float + std::str::FromStr>(
        frequency_string: &str,
    ) -> Option<F> {
        let trimmed_string = frequency_string.trim_end_matches(&[' ', 'H', 'z']);
        trimmed_string
            .parse::<F>()
            .ok()
            .map(utils::frequency_to_log)
    }

    fn log_frequency_formatter(
        mark: egui_plot::GridMark,
        _range: &std::ops::RangeInclusive<f64>,
    ) -> String {
        let log_frequency = mark.value;
        if log_frequency.fract().abs() < 1e-6 {
            let frequency = utils::log_to_frequency(log_frequency);
            format!("{}", frequency)
        } else {
            String::new()
        }
    }
}
