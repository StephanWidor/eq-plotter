use std::ops::RangeInclusive;

use audio_lib::*;

pub fn range_to_f64<F: utils::Float>(range: &RangeInclusive<F>) -> RangeInclusive<f64> {
    range.start().to_f64().unwrap()..=range.end().to_f64().unwrap()
}

pub fn make_log_frequency_points<'a, F: utils::Float>(
    frequency_response: impl Fn(F) -> F + 'a,
    log_frequency_range: &RangeInclusive<F>,
) -> egui_plot::PlotPoints<'a> {
    egui_plot::PlotPoints::from_explicit_callback(
        move |log_frequency| {
            frequency_response(F::from(utils::log_to_frequency(log_frequency)).unwrap())
                .to_f64()
                .unwrap()
        },
        range_to_f64(log_frequency_range),
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

pub fn log_frequency_formatter<F: utils::Float>(
    mark: egui_plot::GridMark,
    _: &std::ops::RangeInclusive<F>,
) -> String {
    let log_frequency = mark.value;
    if log_frequency.fract().abs() < 1e-6 {
        let frequency = utils::log_to_frequency(log_frequency);
        format!("{}", frequency)
    } else {
        String::new()
    }
}
