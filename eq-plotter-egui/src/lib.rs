use app_lib as app;
use audio_lib::biquad;
use audio_lib::eq;
use audio_lib::utils;
use enum_table::Enumable;
use num::complex::ComplexFloat;
use num_traits::Pow;
use std::sync::atomic;

#[derive(Clone, Copy, enum_table::Enumable)]
pub enum ShowOptionType {
    Gain,
    Phase,
    ImpulseResponse,
    PolesAndZeros,
}

pub struct ShowOptions {
    options: [atomic::AtomicBool; ShowOptionType::COUNT],
}

impl Default for ShowOptions {
    fn default() -> Self {
        Self {
            options: [
                atomic::AtomicBool::new(true),
                atomic::AtomicBool::new(false),
                atomic::AtomicBool::new(false),
                atomic::AtomicBool::new(false),
            ],
        }
    }
}

impl ShowOptions {
    pub fn load_relaxed(&self, option_type: ShowOptionType) -> bool {
        self.options[option_type as usize].load(atomic::Ordering::Relaxed)
    }

    pub fn store_relaxed(&self, option_type: ShowOptionType, show: bool) {
        self.options[option_type as usize].store(show, atomic::Ordering::Relaxed);
    }
}

pub struct EqPlotter {
    eqs: Vec<eq::Eq<f64>>,
    sample_rate: f64,
    show_options: ShowOptions,
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
                EqPlotter::draw(ui, &mut self.eqs, self.sample_rate, &mut self.show_options);
            });
    }
}

impl EqPlotter {
    pub const WINDOW_SIZE: [u32; 2] = [1250, 1000]; // [width, height]
    const EQ_COLORS: [egui::Color32; 8] = [
        egui::Color32::from_rgb(140, 51, 51),
        egui::Color32::from_rgb(140, 107, 54),
        egui::Color32::from_rgb(104, 140, 56),
        egui::Color32::from_rgb(59, 140, 106),
        egui::Color32::from_rgb(59, 102, 140),
        egui::Color32::from_rgb(77, 58, 140),
        egui::Color32::from_rgb(140, 60, 140),
        egui::Color32::from_rgb(140, 82, 99),
    ];
    const MULTI_BAND_COLOR: egui::Color32 = egui::Color32::LIGHT_GRAY;
    pub const START_EQ: eq::Eq<f64> = eq::Eq {
        gain: eq::Gain::Db(0.0),
        frequency: eq::Frequency::Hz(1000.0),
        q: 0.7,
        eq_type: eq::EqType::Bypassed,
    };

    pub fn new(num_bands: usize) -> Self {
        assert!(num_bands > 0);
        let mut eq_plotter = Self {
            eqs: vec![Self::START_EQ; num_bands],
            sample_rate: 48000.0,
            show_options: ShowOptions::default(),
        };
        eq_plotter.eqs[0] = app::DEFAULT_EQ;
        eq_plotter
    }

    pub fn draw(
        ui: &mut egui::Ui,
        eqs: &mut [eq::Eq<f64>],
        sample_rate: f64,
        show_options: &ShowOptions,
    ) {
        let ui_size = ui.available_size();

        ui.horizontal(|ui| {
            let mut show_gain = show_options.load_relaxed(ShowOptionType::Gain);
            let mut show_phase = show_options.load_relaxed(ShowOptionType::Phase);
            let mut show_impulse_response =
                show_options.load_relaxed(ShowOptionType::ImpulseResponse);
            let mut show_poles_and_zeros = show_options.load_relaxed(ShowOptionType::PolesAndZeros);

            let control_width = 230_f32;
            let control_outer_margin = 10_f32;
            egui::Frame::group(ui.style()).show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .min_scrolled_height(ui_size.y)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            egui::CollapsingHeader::new("Show").show(ui, |ui| {
                                ui.checkbox(&mut show_gain, "Gain");
                                ui.checkbox(&mut show_phase, "Phase");
                                ui.checkbox(&mut show_impulse_response, "Impulse Response");
                                ui.checkbox(&mut show_poles_and_zeros, "Poles And Zeros");
                            });
                            for (index, eq) in eqs.iter_mut().enumerate() {
                                Self::eq_control(
                                    ui,
                                    control_width,
                                    control_outer_margin,
                                    Self::EQ_COLORS[index % Self::EQ_COLORS.len()],
                                    eq,
                                );
                            }
                        });
                    });
            });

            show_options.store_relaxed(ShowOptionType::Gain, show_gain);
            show_options.store_relaxed(ShowOptionType::Phase, show_phase);
            show_options.store_relaxed(ShowOptionType::ImpulseResponse, show_impulse_response);
            show_options.store_relaxed(ShowOptionType::PolesAndZeros, show_poles_and_zeros);

            if !(show_gain || show_phase || show_impulse_response || show_poles_and_zeros) {
                return;
            }

            let num_rows = (((show_gain && show_phase)
                || (show_impulse_response && show_poles_and_zeros))
                as usize
                + 1) as f32;
            let num_columns = (((show_gain || show_phase)
                && (show_impulse_response || show_poles_and_zeros))
                as usize
                + 1) as f32;
            let available_size = egui::Vec2::new(
                ui_size.x - control_width - 2_f32 * control_outer_margin - 20_f32,
                ui_size.y,
            );
            let plot_size = (available_size.x / num_columns).min(available_size.y / num_rows);

            if plot_size < 50_f32 {
                return;
            }

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

            egui::Frame::group(ui.style())
                .outer_margin(0_f32)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                if show_gain {
                                    Self::gain_plot(
                                        ui,
                                        &frequency_responses,
                                        &active_eqs,
                                        &multiband_frequency_response,
                                        plot_size,
                                    );
                                }
                                if show_phase {
                                    Self::phase_plot(
                                        ui,
                                        &frequency_responses,
                                        &active_eqs,
                                        &multiband_frequency_response,
                                        plot_size,
                                    );
                                }
                            });

                            ui.vertical(|ui| {
                                if show_impulse_response {
                                    let (impulse_responses, multiband_impulse_response) =
                                        Self::impulse_responses(&coefficients);
                                    Self::impulse_response_plot(
                                        ui,
                                        &impulse_responses,
                                        &active_eqs,
                                        &multiband_impulse_response,
                                        plot_size,
                                    );
                                }
                                if show_poles_and_zeros {
                                    Self::poles_and_zeros_plot(
                                        ui,
                                        &coefficients,
                                        &active_eqs,
                                        plot_size,
                                    );
                                }
                            });
                        });
                    });
                });
        });
    }

    fn eq_control(
        ui: &mut egui::Ui,
        width: f32,
        outer_margin: f32,
        color: egui::Color32,
        eq: &mut eq::Eq<f64>,
    ) {
        let mut gain_db = eq.gain.db();
        let mut log_frequency = eq.frequency.log_hz();
        egui::Frame::group(ui.style())
            .fill(color)
            .multiply_with_opacity(0.2_f32)
            .corner_radius(5)
            .outer_margin(outer_margin)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    egui::ComboBox::from_id_salt(ui.next_auto_id())
                        .selected_text(eq.eq_type.to_string())
                        .width(width)
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
            .width(Self::effective_plot_size(plot_size))
            .height(Self::effective_plot_size(plot_size))
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
            .width(Self::effective_plot_size(plot_size))
            .height(Self::effective_plot_size(plot_size))
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

    fn impulse_responses(
        coefficients: &[biquad::coefficients::Coefficients<f64>],
    ) -> (Vec<Vec<f64>>, Vec<f64>) {
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
        (impulse_responses, multiband_impulse_response)
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
            .width(Self::effective_plot_size(plot_size))
            .height(Self::effective_plot_size(plot_size))
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
            .width(Self::effective_plot_size(plot_size))
            .height(Self::effective_plot_size(plot_size))
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

    // Seems like labels etc are not in width and height we give to Plot::new.
    // Not nice, but then let's just give it width and height made a bit smaller
    fn effective_plot_size(plot_size: f32) -> f32 {
        plot_size - 15_f32
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
