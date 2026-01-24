use app_lib as app;
use audio_lib::*;

pub fn make_log_frequency_points<'a>(
    frequency_response: impl Fn(f64) -> f64 + 'a,
) -> egui_plot::PlotPoints<'a> {
    egui_plot::PlotPoints::from_explicit_callback(
        move |log_frequency| frequency_response(utils::log_to_frequency(log_frequency)),
        app::MIN_LOG_FREQUENCY..=app::MAX_LOG_FREQUENCY,
        1000,
    )
}

pub fn make_circle_points<'a>(radius: f64, num_points: usize) -> egui_plot::PlotPoints<'a> {
    let circle_point = move |angle: f64| (radius * angle.cos(), radius * angle.sin());
    egui_plot::PlotPoints::from_parametric_callback(
        circle_point,
        0.0..=2.0 * std::f64::consts::PI,
        num_points,
    )
}

pub fn impulse_responses(
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
    let multiband_impulse_response = biquad::utils::multiband::impulse_response_for_coefficients(
        &coefficients,
        eps_for_impulse,
        hold_for_impulse,
        max_length_for_impulse,
    );
    (impulse_responses, multiband_impulse_response)
}

pub fn log_frequency_to_string<F: utils::Float + std::fmt::Display>(log_frequency: F) -> String {
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

pub fn log_frequency_formatter(
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
